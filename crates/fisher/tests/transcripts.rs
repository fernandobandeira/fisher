#[path = "support/principal_variation.rs"]
mod principal_variation;
#[path = "support/reference_minimax.rs"]
mod reference;
#[path = "support/scripted_transcripts.rs"]
mod scripted_transcripts;

use std::{
    fs,
    io::{self, BufRead, BufReader, Write},
    path::Path,
    process::{Child, Command, Stdio},
    sync::{
        Mutex,
        mpsc::{self, Receiver},
    },
    thread,
    time::{Duration, Instant},
};

use shakmaty::{CastlingMode, Chess, Move, Position, fen::Fen, uci::UciMove};

const LEGAL_BESTMOVE: &str = "bestmove {legal}";
const TIMEOUT: Duration = Duration::from_secs(5);

#[test]
fn handshake() -> io::Result<()> {
    play_transcript("handshake")
}

#[test]
fn unknown_input() -> io::Result<()> {
    play_transcript("unknown-input")
}

#[test]
fn ignored_commands() -> io::Result<()> {
    play_transcript("ignored-commands")
}

#[test]
fn quit() -> io::Result<()> {
    play_transcript("quit")
}

#[test]
fn end_of_input() -> io::Result<()> {
    play_transcript("end-of-input")
}

#[test]
fn startpos() -> io::Result<()> {
    play_transcript("startpos")
}

#[test]
fn new_game() -> io::Result<()> {
    play_transcript("new-game")
}

#[test]
fn position_moves() -> io::Result<()> {
    play_transcript("position-moves")
}

#[test]
fn go_parameters() -> io::Result<()> {
    play_transcript("go-parameters")
}

#[test]
fn terminal_positions() -> io::Result<()> {
    play_transcript("terminal-positions")
}

#[test]
fn halfmove_clock() -> io::Result<()> {
    play_transcript("halfmove-clock")
}

#[test]
fn rejected_position() -> io::Result<()> {
    play_transcript("rejected-position")
}

#[test]
fn rejected_position_opposite_check() -> io::Result<()> {
    play_transcript("rejected-position-opposite-check")
}

#[test]
fn go_infinite() -> io::Result<()> {
    play_transcript("go-infinite")
}

#[test]
fn go_infinite_terminal() -> io::Result<()> {
    play_transcript("go-infinite-terminal")
}

#[test]
fn quit_while_held() -> io::Result<()> {
    play_transcript("quit-while-held")
}

#[test]
fn end_of_input_while_held() -> io::Result<()> {
    play_transcript("end-of-input-while-held")
}

#[test]
fn mate_in_one() -> io::Result<()> {
    play_transcript("mate-in-one")
}

#[test]
fn material_zero() -> io::Result<()> {
    play_transcript("material-zero")
}

#[test]
fn equal_scores() -> io::Result<()> {
    let first = collect_legal_bestmove_replies("equal-scores")?;
    let second = collect_legal_bestmove_replies("equal-scores")?;
    assert_eq!(first.len(), 1);
    assert_eq!(first, second);
    Ok(())
}

fn play_transcript(name: &str) -> io::Result<()> {
    collect_legal_bestmove_replies(name).map(drop)
}

fn collect_legal_bestmove_replies(name: &str) -> io::Result<Vec<String>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../expectations/uci")
        .join(format!("{name}.uci"));
    let transcript = fs::read_to_string(path)?;
    run_transcript(name, &transcript, Engine::start)
}

fn run_transcript<I: TranscriptIo>(
    name: &str,
    transcript: &str,
    start: impl FnOnce() -> io::Result<I>,
) -> io::Result<Vec<String>> {
    let records = load_transcript(name, transcript)?;
    Session::new(start()?).play(name, records)
}

enum Delimiter<'a> {
    Reply(&'a str),
    Exit,
}

impl<'a> Delimiter<'a> {
    fn parse(line: &'a str) -> Option<Self> {
        match line {
            "! exit" => Some(Self::Exit),
            _ => line.strip_prefix("< ").map(Self::Reply),
        }
    }
}

enum Record<'a> {
    Send(Vec<u8>),
    CloseInput,
    Observe(Option<SearchInfo<'a>>, Delimiter<'a>),
}

fn record_error(name: &str, index: usize, error: impl std::fmt::Display) -> io::Error {
    io::Error::other(format!("{name}.uci:{}: {error}", index + 1))
}

fn load_transcript<'a>(name: &str, transcript: &'a str) -> io::Result<Vec<(usize, Record<'a>)>> {
    let mut lines = transcript
        .lines()
        .enumerate()
        .filter(|&(_, line)| !line.is_empty() && !line.starts_with('#'));
    let mut records = Vec::new();
    while let Some((index, line)) = lines.next() {
        let error = |message: &str| record_error(name, index, message);
        let record = if let Some(pattern) = line.strip_prefix("<* ") {
            let pattern = search_info(pattern)
                .filter(|info| info.valid(true))
                .ok_or_else(|| error("invalid repeated searched info pattern"))?;
            let delimiter = lines
                .next()
                .and_then(|(_, line)| Delimiter::parse(line))
                .ok_or_else(|| error("repeated searched info requires a reply or ! exit"))?;
            Record::Observe(Some(pattern), delimiter)
        } else if let Some(delimiter) = Delimiter::parse(line) {
            Record::Observe(None, delimiter)
        } else if let Some((command, ending)) = line
            .strip_prefix("> ")
            .map(|command| (command, b'\n'))
            .or_else(|| line.strip_prefix(">CR ").map(|command| (command, b'\r')))
        {
            let mut command =
                decode_command(command).map_err(|error| record_error(name, index, error))?;
            command.push(ending);
            Record::Send(command)
        } else if line == "! eof" {
            Record::CloseInput
        } else {
            return Err(record_error(
                name,
                index,
                format!("unknown record {line:?}"),
            ));
        };
        let exit = matches!(record, Record::Observe(_, Delimiter::Exit));
        records.push((index, record));
        if exit {
            return match lines.next() {
                Some((index, _)) => Err(record_error(name, index, "record after ! exit")),
                None => Ok(records),
            };
        }
    }
    Err(io::Error::other(format!("{name}.uci: missing ! exit")))
}

fn decode_command(command: &str) -> io::Result<Vec<u8>> {
    let mut decoded = Vec::with_capacity(command.len());
    let mut bytes = command.bytes();
    while let Some(byte) = bytes.next() {
        decoded.push(if byte == b'\\' {
            match bytes.next() {
                Some(b't') => b'\t',
                Some(b'r') => b'\r',
                Some(b'\\') => b'\\',
                Some(b'x') => {
                    let high = decode_hex_digit(bytes.next())?;
                    let low = decode_hex_digit(bytes.next())?;
                    high * 16 + low
                }
                other => {
                    return Err(io::Error::other(format!(
                        "invalid escape {:?}",
                        other.map(char::from)
                    )));
                }
            }
        } else {
            byte
        });
    }
    Ok(decoded)
}

fn decode_hex_digit(byte: Option<u8>) -> io::Result<u8> {
    match byte {
        Some(byte @ b'0'..=b'9') => Ok(byte - b'0'),
        Some(byte @ b'a'..=b'f') => Ok(byte - b'a' + 10),
        Some(byte @ b'A'..=b'F') => Ok(byte - b'A' + 10),
        _ => Err(io::Error::other("invalid hex escape")),
    }
}

fn reference_command<'a>(tokens: &mut impl Iterator<Item = &'a str>) -> Option<&'a str> {
    tokens.find(|&token| {
        matches!(
            token,
            "uci"
                | "isready"
                | "ucinewgame"
                | "position"
                | "go"
                | "quit"
                | "stop"
                | "setoption"
                | "debug"
                | "ponderhit"
                | "register"
        )
    })
}

fn reference_position<'a>(mut tokens: impl Iterator<Item = &'a str>) -> Option<Chess> {
    let mut position = match tokens.next()? {
        "startpos" => Chess::default(),
        "fen" => {
            let fields: Vec<_> = tokens.by_ref().take(6).collect();
            if fields.len() != 6 {
                return None;
            }
            fields
                .join(" ")
                .parse::<Fen>()
                .ok()?
                .into_position(CastlingMode::Standard)
                .ok()?
        }
        _ => return None,
    };
    match tokens.next() {
        None => return Some(position),
        Some("moves") => {}
        Some(_) => return None,
    }
    for token in tokens {
        let movement = reference_move(&position, token)?;
        position.play_unchecked(movement);
    }
    Some(position)
}

fn reference_move(position: &Chess, text: &str) -> Option<Move> {
    position
        .legal_moves()
        .into_iter()
        .find(|&movement| UciMove::from_standard(movement).to_string() == text)
}

fn reply_matches(expected: &str, actual: &str, position: &Chess) -> bool {
    match expected {
        "id name fisher {version}" => {
            actual == concat!("id name fisher ", env!("CARGO_PKG_VERSION"))
        }
        LEGAL_BESTMOVE => actual
            .strip_prefix("bestmove ")
            .is_some_and(|text| reference_move(position, text).is_some()),
        // The README makes the full spelling special for a single `< ` record at
        // any literal positive depth, and the short spelling at depth 4 only;
        // `{depth}` belongs to `<* ` alone.
        _ if search_info(expected).is_some_and(|info| {
            info.valid(true)
                && positive_decimal(info.depth)
                && (info.full.is_some() || info.depth == "4")
        }) =>
        {
            search_info_matches(expected, actual)
        }
        "info string {message}" => actual.starts_with("info string "),
        _ if let Some(required) = expected
            .strip_prefix("info string {message:")
            .and_then(|text| text.strip_suffix('}')) =>
        {
            actual
                .strip_prefix("info string ")
                .is_some_and(|message| message.contains(required))
        }
        _ => expected == actual,
    }
}

/// The deepest search the reference minimax checks; it prunes nothing.
const REFERENCE_DEPTH: u8 = 4;

#[test]
fn single_replies_keep_allowances_to_literal_depths() {
    let position = Chess::default();
    let cases = [
        (
            "info depth {depth} score cp 0 nodes 20",
            "info depth 1 score cp 0 nodes 20",
        ),
        (
            "info depth 3 score {score} nodes {nodes}",
            "info depth 3 score cp 0 nodes 8902",
        ),
        (
            "info depth {depth} multipv 1 score {score} nodes {nodes} nps {nps} time {time} pv {pv}",
            "info depth 1 multipv 1 score cp 0 nodes 20 nps 20000 time 1 pv e2e4",
        ),
    ];
    for (expected, actual) in cases {
        assert!(
            !reply_matches(expected, actual, &position),
            "{expected:?} must compare exactly"
        );
    }
    assert!(reply_matches(
        "info depth 4 score {score} nodes {nodes}",
        "info depth 4 score cp 0 nodes 206603",
        &position
    ));
    for (expected, actual) in [
        (
            "info depth 1 multipv 1 score cp 0 nodes 20 nps {nps} time {time} pv {pv}",
            "info depth 1 multipv 1 score cp 0 nodes 20 nps 20000 time 1 pv g1h3",
        ),
        (
            "info depth 7 multipv 1 score {score} nodes {nodes} nps {nps} time {time} pv {pv}",
            "info depth 7 multipv 1 score cp 0 nodes 1 nps 1000 time 1 pv e2e4",
        ),
    ] {
        assert!(
            reply_matches(expected, actual, &position),
            "{expected:?} must accept {actual:?}"
        );
    }
}

#[test]
fn info_allowance_matches_the_documented_grammar() {
    let expected =
        "info depth 4 multipv 1 score {score} nodes {nodes} nps {nps} time {time} pv {pv}";
    // The allowance pattern in expectations/README.md sets these results.
    let cases = [
        ("cp 0 nodes 0 nps 0 time 1 pv a7a8q", true),
        (
            "cp 100 nodes 197281 nps 197281000 time 1 pv b1a3 b8a6",
            true,
        ),
        ("cp -100 nodes 20 nps 20000 time 1 pv b1a3", true),
        ("mate 2 nodes 20 nps 20000 time 1 pv b1a3", true),
        (
            "mate -2 nodes 20 nps 20000 time 1 pv b1a3 b8a6 g1f3 g8f6",
            true,
        ),
        ("cp 0 nodes 020 nps 20000 time 1 pv b1a3", false),
        ("cp +-100 nodes 20 nps 20000 time 1 pv b1a3", false),
        ("mate 0 nodes 20 nps 20000 time 1 pv b1a3", false),
        ("cp 0 nodes 20 nps 20000 time 01 pv b1a3", false),
        ("cp 0 nodes 20 nps -1 time 1 pv b1a3", false),
        ("cp 0 nodes 20 nps 20000 time 1 pv 0000", false),
        ("cp 0 nodes 20 nps 20000 time 1 pv a7a8K", false),
        ("cp 0 nodes 20 nps 20000 time 1 pv b1a3 ", false),
        ("cp 0 nodes 20 nps 20000 time 1 pv b1a3  b8a6", false),
        (
            "cp 0 nodes 20 nps 20000 time 1 pv b1a3 b8a6 g1f3 g8f6 a2a3",
            false,
        ),
    ];

    for (suffix, matches) in cases {
        let actual = format!("info depth 4 multipv 1 score {suffix}");
        assert_eq!(search_info_matches(expected, &actual), matches, "{actual}");
    }
}

struct SearchInfo<'a> {
    depth: &'a str,
    score: &'a str,
    nodes: &'a str,
    full: Option<FullSearchInfo<'a>>,
}

struct FullSearchInfo<'a> {
    time: &'a str,
    nps: &'a str,
    pv: &'a str,
}

fn search_info(text: &str) -> Option<SearchInfo<'_>> {
    let (depth, rest) = text.strip_prefix("info depth ")?.split_once(" score ")?;
    let (score, rest) = rest.split_once(" nodes ")?;
    let (depth, nodes, full) = if let Some(depth) = depth.strip_suffix(" multipv 1") {
        let (nodes, rest) = rest.split_once(" nps ")?;
        let (nps, rest) = rest.split_once(" time ")?;
        let (time, pv) = rest.split_once(" pv ")?;
        (depth, nodes, Some(FullSearchInfo { time, nps, pv }))
    } else {
        (depth, rest, None)
    };
    Some(SearchInfo {
        depth,
        score,
        nodes,
        full,
    })
}

fn verify_node_rate(info: &SearchInfo<'_>) -> io::Result<()> {
    let Some(full) = &info.full else {
        return Ok(());
    };
    let nodes = info.nodes.parse::<u64>().map_err(io::Error::other)?;
    let time = full.time.parse::<u64>().map_err(io::Error::other)?;
    let nps = full.nps.parse::<u64>().map_err(io::Error::other)?;
    let expected = (u128::from(nodes) * 1000)
        .checked_div(u128::from(time))
        .ok_or_else(|| io::Error::other("searched info reported time 0"))?;
    if u128::from(nps) != expected {
        return Err(io::Error::other(format!(
            "reported nps {nps}, expected {expected} from nodes {nodes} and time {time}"
        )));
    }
    Ok(())
}

impl SearchInfo<'_> {
    fn valid(&self, pattern: bool) -> bool {
        let slot = |text, allowance| pattern && text == allowance;
        (slot(self.depth, "{depth}") || positive_decimal(self.depth))
            && (slot(self.score, "{score}") || score_matches(self.score))
            && (slot(self.nodes, "{nodes}") || nonnegative_decimal(self.nodes))
            && self.full.as_ref().is_none_or(|full| {
                (slot(full.time, "{time}") || positive_decimal(full.time))
                    && (slot(full.nps, "{nps}") || nonnegative_decimal(full.nps))
                    && (slot(full.pv, "{pv}")
                        || (full.pv.split(' ').all(uci_move_matches)
                            && full.pv.split(' ').count()
                                <= self.depth.parse().unwrap_or(usize::MAX)))
            })
    }

    fn matches(&self, actual: &Self) -> bool {
        let slot = |expected, allowance, actual| expected == allowance || expected == actual;
        actual.valid(false)
            && slot(self.depth, "{depth}", actual.depth)
            && slot(self.score, "{score}", actual.score)
            && slot(self.nodes, "{nodes}", actual.nodes)
            && match (&self.full, &actual.full) {
                (None, None) => true,
                (Some(expected), Some(actual)) => {
                    slot(expected.time, "{time}", actual.time)
                        && slot(expected.nps, "{nps}", actual.nps)
                        && slot(expected.pv, "{pv}", actual.pv)
                }
                _ => false,
            }
    }
}

fn search_info_matches(expected: &str, actual: &str) -> bool {
    search_info(expected)
        .zip(search_info(actual))
        .is_some_and(|(expected, actual)| expected.matches(&actual))
}

fn score_matches(score: &str) -> bool {
    if let Some(cp) = score.strip_prefix("cp ") {
        nonnegative_decimal(cp) || positive_decimal(cp.strip_prefix('-').unwrap_or(cp))
    } else if let Some(mate) = score.strip_prefix("mate ") {
        positive_decimal(mate.strip_prefix('-').unwrap_or(mate))
    } else {
        false
    }
}

fn nonnegative_decimal(text: &str) -> bool {
    text == "0" || positive_decimal(text)
}

fn uci_move_matches(text: &str) -> bool {
    let ([file_from, rank_from, file_to, rank_to]
    | [
        file_from,
        rank_from,
        file_to,
        rank_to,
        b'q' | b'r' | b'b' | b'n',
    ]) = text.as_bytes()
    else {
        return false;
    };
    (b'a'..=b'h').contains(file_from)
        && (b'a'..=b'h').contains(file_to)
        && (b'1'..=b'8').contains(rank_from)
        && (b'1'..=b'8').contains(rank_to)
}

fn reported_score(text: &str) -> io::Result<i32> {
    if let Some(cp) = text.strip_prefix("cp ") {
        return cp.parse().map_err(io::Error::other);
    }
    let mate = text
        .strip_prefix("mate ")
        .ok_or_else(|| io::Error::other("missing score"))?
        .parse::<i32>()
        .map_err(io::Error::other)?;
    // At depth four only mates in one or two can be reported.
    if !(1..=2).contains(&mate.unsigned_abs()) {
        return Err(io::Error::other(format!(
            "mate {mate} exceeds the search depth"
        )));
    }
    let plies = 2 * mate.abs() - i32::from(mate > 0);
    Ok(mate.signum() * (reference::MATE_SCORE - plies))
}

fn positive_decimal(text: &str) -> bool {
    matches!(text.as_bytes().first(), Some(b'1'..=b'9'))
        && text.bytes().all(|byte| byte.is_ascii_digit())
}

trait TranscriptIo {
    fn send(&mut self, command: &[u8]) -> io::Result<()>;
    fn close_input(&mut self);
    fn now(&self) -> Instant;
    fn read_reply(&mut self, deadline: Instant) -> io::Result<Option<String>>;
    fn wait_for_success(&mut self, deadline: Instant) -> io::Result<()>;
}

struct Engine {
    child: Child,
    output: Receiver<io::Result<String>>,
}

impl Engine {
    fn start() -> io::Result<Self> {
        let child = {
            // On macOS, concurrent spawns can inherit each other's pipes and delay EOF.
            static SPAWN: Mutex<()> = Mutex::new(());
            let _guard = SPAWN
                .lock()
                .map_err(|_| io::Error::other("fisher spawn lock is poisoned"))?;
            Command::new(env!("CARGO_BIN_EXE_fisher"))
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()?
        };
        let (sender, output) = mpsc::channel();
        let mut engine = Self { child, output };
        let stdout = engine
            .child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("missing fisher stdout"))?;

        thread::Builder::new().spawn(move || {
            for line in BufReader::new(stdout).lines() {
                if sender.send(line).is_err() {
                    break;
                }
            }
        })?;

        Ok(engine)
    }
}

impl TranscriptIo for Engine {
    fn send(&mut self, command: &[u8]) -> io::Result<()> {
        let stdin = self
            .child
            .stdin
            .as_mut()
            .ok_or_else(|| io::Error::other("fisher stdin is closed"))?;
        stdin.write_all(command)?;
        stdin.flush()
    }

    fn close_input(&mut self) {
        drop(self.child.stdin.take());
    }

    fn now(&self) -> Instant {
        Instant::now()
    }

    fn read_reply(&mut self, deadline: Instant) -> io::Result<Option<String>> {
        let remaining = deadline.saturating_duration_since(self.now());
        if remaining.is_zero() {
            return Err(read_timeout());
        }
        let result = self.output.recv_timeout(remaining);
        if self.now() >= deadline {
            return Err(read_timeout());
        }
        match result {
            Ok(line) => line.map(Some),
            Err(mpsc::RecvTimeoutError::Disconnected) => Ok(None),
            Err(mpsc::RecvTimeoutError::Timeout) => Err(read_timeout()),
        }
    }

    fn wait_for_success(&mut self, deadline: Instant) -> io::Result<()> {
        while self.now() < deadline {
            if let Some(status) = self.child.try_wait()? {
                return if status.success() {
                    Ok(())
                } else {
                    Err(io::Error::other(format!("fisher exited with {status}")))
                };
            }
            thread::sleep(Duration::from_millis(5));
        }
        Err(exit_timeout())
    }
}

fn read_timeout() -> io::Error {
    io::Error::new(io::ErrorKind::TimedOut, "timed out reading fisher stdout")
}

fn exit_timeout() -> io::Error {
    io::Error::new(
        io::ErrorKind::TimedOut,
        "timed out waiting for fisher to exit",
    )
}

struct Session<I> {
    io: I,
    position: Chess,
    holding: bool,
    first_variation_move: Option<String>,
    legal_bestmove_replies: Vec<String>,
}

impl<I: TranscriptIo> Session<I> {
    fn new(io: I) -> Self {
        Self {
            io,
            position: Chess::default(),
            holding: false,
            first_variation_move: None,
            legal_bestmove_replies: Vec::new(),
        }
    }

    fn play(&mut self, name: &str, records: Vec<(usize, Record<'_>)>) -> io::Result<Vec<String>> {
        for (index, record) in records {
            self.play_record(record).map_err(|error| {
                io::Error::new(error.kind(), format!("{name}.uci:{}: {error}", index + 1))
            })?;
        }
        Ok(std::mem::take(&mut self.legal_bestmove_replies))
    }

    fn play_record(&mut self, record: Record<'_>) -> io::Result<()> {
        match record {
            Record::Send(command) => {
                self.io.send(&command)?;
                self.track_reference_command(&command);
                Ok(())
            }
            Record::CloseInput => {
                self.io.close_input();
                Ok(())
            }
            Record::Observe(pattern, delimiter) => self.observe(pattern.as_ref(), delimiter),
        }
    }

    fn observe(
        &mut self,
        pattern: Option<&SearchInfo<'_>>,
        delimiter: Delimiter<'_>,
    ) -> io::Result<()> {
        let deadline = self.io.now() + TIMEOUT;
        let actual = loop {
            let actual = self.io.read_reply(deadline)?;
            let info = actual.as_deref().and_then(search_info);
            if let Some(info) = &info {
                verify_node_rate(info)?;
            }
            if let Some((pattern, info)) = pattern.zip(info.as_ref())
                && pattern.matches(info)
            {
                self.verify_search_info(pattern.score, info)?;
            } else {
                break actual;
            }
        };
        match (delimiter, actual) {
            (Delimiter::Reply(expected), Some(actual)) => self.expect_reply(expected, actual),
            (Delimiter::Reply(expected), None) => Err(io::Error::other(format!(
                "stdout closed while expecting {expected:?}"
            ))),
            (Delimiter::Exit, Some(extra)) => {
                Err(io::Error::other(format!("unexpected output {extra:?}")))
            }
            (Delimiter::Exit, None) => self.io.wait_for_success(self.io.now() + TIMEOUT),
        }
    }

    fn track_reference_command(&mut self, command: &[u8]) {
        let text = String::from_utf8_lossy(command);
        let mut tokens = text.split_whitespace();
        match reference_command(&mut tokens) {
            Some("go") if tokens.any(|token| token == "infinite") => self.holding = true,
            Some("stop") => self.holding = false,
            Some("position") if !self.holding => {
                if let Some(position) = reference_position(tokens) {
                    self.position = position;
                }
            }
            _ => {}
        }
    }

    fn expect_reply(&mut self, expected: &str, actual: String) -> io::Result<()> {
        if !reply_matches(expected, &actual, &self.position) {
            return Err(io::Error::other(format!(
                "expected {expected:?}, got {actual:?}"
            )));
        }
        if let Some((pattern, info)) = search_info(expected).zip(search_info(&actual)) {
            self.verify_search_info(pattern.score, &info)?;
        }
        if let Some(bestmove) = actual.strip_prefix("bestmove ")
            && let Some(first) = self.first_variation_move.take()
            && first != bestmove
        {
            return Err(io::Error::other(format!(
                "principal variation starts with {first}, got bestmove {bestmove}"
            )));
        }
        if expected == LEGAL_BESTMOVE {
            self.legal_bestmove_replies.push(actual);
        }
        Ok(())
    }

    fn verify_search_info(
        &mut self,
        expected_score: &str,
        info: &SearchInfo<'_>,
    ) -> io::Result<()> {
        // Depths 1 to 4 have an independent minimax and replay check. Deeper
        // lines have the documented spelling and length checks only, because
        // the reference minimax prunes nothing.
        let Some((full, depth)) = info.full.as_ref().zip(
            info.depth
                .parse::<u8>()
                .ok()
                .filter(|depth| (1..=REFERENCE_DEPTH).contains(depth)),
        ) else {
            self.first_variation_move = None;
            return Ok(());
        };
        let score = reported_score(info.score)?;
        if expected_score == "{score}" {
            let best = reference::minimax(&self.position, depth).score;
            if score != best {
                return Err(io::Error::other(format!(
                    "reported score {score} differs from reference {best}"
                )));
            }
        }
        principal_variation::verify(&self.position, depth, score, full.pv.split(' '))
            .map_err(io::Error::other)?;
        self.first_variation_move = full.pv.split(' ').next().map(str::to_owned);
        Ok(())
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        drop(self.child.kill());
        drop(self.child.wait());
    }
}
