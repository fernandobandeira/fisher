#[path = "support/principal_variation.rs"]
mod principal_variation;
#[path = "support/reference_minimax.rs"]
mod reference;

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
    let directory = match name {
        "end-of-input-while-held"
        | "equal-scores"
        | "go-infinite"
        | "go-parameters"
        | "halfmove-clock"
        | "mate-in-one"
        | "material-zero"
        | "new-game"
        | "position-moves"
        | "quit-while-held"
        | "rejected-position-opposite-check"
        | "rejected-position"
        | "startpos" => "uci-principal-variation",
        _ => "uci",
    };
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../expectations")
        .join(directory)
        .join(format!("{name}.uci"));
    let transcript = fs::read_to_string(path)?;
    let mut session = Session::start()?;
    let mut ended = false;

    for (line_index, line) in transcript.lines().enumerate() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        ended = if ended {
            Err(io::Error::other("record after ! exit"))
        } else {
            session.play_record(line)
        }
        .map_err(|error| io::Error::other(format!("{name}.uci:{}: {error}", line_index + 1)))?;
    }

    if ended {
        Ok(std::mem::take(&mut session.legal_bestmove_replies))
    } else {
        Err(io::Error::other(format!("{name}.uci: missing ! exit")))
    }
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
        _ if expected.starts_with("info depth 4 score ") => search_info_matches(expected, actual),
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

#[test]
fn info_allowance_matches_the_documented_grammar() {
    let expected = "info depth 4 score {score} time {time} nodes {nodes} nps {nps} pv {pv}";
    // The allowance pattern in expectations/README.md sets these results.
    let cases = [
        ("cp 0 time 0 nodes 0 nps 0 pv a7a8q", true),
        (
            "cp 100 time 1 nodes 197281 nps 197281000 pv b1a3 b8a6",
            true,
        ),
        ("cp -100 time 1 nodes 20 nps 20000 pv b1a3", true),
        ("mate 2 time 1 nodes 20 nps 20000 pv b1a3", true),
        (
            "mate -2 time 1 nodes 20 nps 20000 pv b1a3 b8a6 g1f3 g8f6",
            true,
        ),
        ("cp 0 time 1 nodes 020 nps 20000 pv b1a3", false),
        ("cp +-100 time 1 nodes 20 nps 20000 pv b1a3", false),
        ("mate 0 time 1 nodes 20 nps 20000 pv b1a3", false),
        ("cp 0 time 01 nodes 20 nps 20000 pv b1a3", false),
        ("cp 0 time 1 nodes 20 nps -1 pv b1a3", false),
        ("cp 0 time 1 nodes 20 nps 20000 pv 0000", false),
        ("cp 0 time 1 nodes 20 nps 20000 pv a7a8K", false),
        ("cp 0 time 1 nodes 20 nps 20000 pv b1a3 ", false),
        ("cp 0 time 1 nodes 20 nps 20000 pv b1a3  b8a6", false),
        (
            "cp 0 time 1 nodes 20 nps 20000 pv b1a3 b8a6 g1f3 g8f6 a2a3",
            false,
        ),
    ];

    for (suffix, matches) in cases {
        let actual = format!("info depth 4 score {suffix}");
        assert_eq!(search_info_matches(expected, &actual), matches, "{actual}");
    }
}

struct SearchInfo<'a> {
    score: &'a str,
    time: &'a str,
    nodes: &'a str,
    nps: &'a str,
    pv: &'a str,
}

fn search_info(text: &str) -> Option<SearchInfo<'_>> {
    let (score, rest) = text
        .strip_prefix("info depth 4 score ")?
        .split_once(" time ")?;
    let (time, rest) = rest.split_once(" nodes ")?;
    let (nodes, rest) = rest.split_once(" nps ")?;
    let (nps, pv) = rest.split_once(" pv ")?;
    Some(SearchInfo {
        score,
        time,
        nodes,
        nps,
        pv,
    })
}

fn search_info_matches(expected: &str, actual: &str) -> bool {
    let (Some(expected), Some(actual)) = (search_info(expected), search_info(actual)) else {
        return false;
    };
    let score_matches = if let Some(cp) = actual.score.strip_prefix("cp ") {
        cp == "0" || positive_decimal(cp.strip_prefix('-').unwrap_or(cp))
    } else if let Some(mate) = actual.score.strip_prefix("mate ") {
        positive_decimal(mate.strip_prefix('-').unwrap_or(mate))
    } else {
        false
    };
    score_matches
        && (expected.score == "{score}" || expected.score == actual.score)
        && (expected.nodes == "{nodes}" || expected.nodes == actual.nodes)
        && [actual.time, actual.nodes, actual.nps]
            .into_iter()
            .all(|text| text == "0" || positive_decimal(text))
        && actual.pv.split(' ').count() <= 4
        && actual.pv.split(' ').all(uci_move_matches)
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

struct Session {
    child: Child,
    output: Receiver<io::Result<String>>,
    position: Chess,
    holding: bool,
    first_variation_move: Option<String>,
    legal_bestmove_replies: Vec<String>,
}

impl Session {
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
        let mut session = Self {
            child,
            output,
            position: Chess::default(),
            holding: false,
            first_variation_move: None,
            legal_bestmove_replies: Vec::new(),
        };
        let stdout = session
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

        Ok(session)
    }

    fn play_record(&mut self, line: &str) -> io::Result<bool> {
        if let Some(command) = line.strip_prefix("> ") {
            self.send(command, b'\n')?;
        } else if let Some(command) = line.strip_prefix(">CR ") {
            self.send(command, b'\r')?;
        } else if let Some(expected) = line.strip_prefix("< ") {
            self.expect_reply(expected)?;
        } else if line == "! eof" {
            drop(self.child.stdin.take());
        } else if line == "! exit" {
            self.expect_exit()?;
            return Ok(true);
        } else {
            return Err(io::Error::other(format!("unknown record {line:?}")));
        }
        Ok(false)
    }

    fn send(&mut self, command: &str, ending: u8) -> io::Result<()> {
        let stdin = self
            .child
            .stdin
            .as_mut()
            .ok_or_else(|| io::Error::other("fisher stdin is closed"))?;
        let command = decode_command(command)?;
        stdin.write_all(&command)?;
        stdin.write_all(&[ending])?;
        stdin.flush()?;
        self.track_reference_command(&command);
        Ok(())
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

    fn expect_reply(&mut self, expected: &str) -> io::Result<()> {
        let actual = self.next_reply()?.ok_or_else(|| {
            io::Error::other(format!("stdout closed while expecting {expected:?}"))
        })?;
        if reply_matches(expected, &actual, &self.position) {
            if let Some(info) = search_info(&actual) {
                let score = reported_score(info.score)?;
                if expected.starts_with("info depth 4 score {score}") {
                    let best = reference::minimax(&self.position, 4).score;
                    if score != best {
                        return Err(io::Error::other(format!(
                            "reported score {score} differs from reference {best}"
                        )));
                    }
                }
                principal_variation::verify(&self.position, 4, score, info.pv.split(' '))
                    .map_err(io::Error::other)?;
                self.first_variation_move = info.pv.split(' ').next().map(str::to_owned);
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
        } else {
            Err(io::Error::other(format!(
                "expected {expected:?}, got {actual:?}"
            )))
        }
    }

    fn next_reply(&self) -> io::Result<Option<String>> {
        match self.output.recv_timeout(TIMEOUT) {
            Ok(line) => line.map(Some),
            Err(mpsc::RecvTimeoutError::Disconnected) => Ok(None),
            Err(error @ mpsc::RecvTimeoutError::Timeout) => Err(io::Error::other(error)),
        }
    }

    fn expect_exit(&mut self) -> io::Result<()> {
        if let Some(extra) = self.next_reply()? {
            return Err(io::Error::other(format!("unexpected output {extra:?}")));
        }
        let deadline = Instant::now() + TIMEOUT;
        loop {
            if let Some(status) = self.child.try_wait()? {
                return if status.success() {
                    Ok(())
                } else {
                    Err(io::Error::other(format!("fisher exited with {status}")))
                };
            }
            if Instant::now() >= deadline {
                return Err(io::Error::other("timed out waiting for fisher to exit"));
            }
            thread::sleep(Duration::from_millis(5));
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        drop(self.child.kill());
        drop(self.child.wait());
    }
}
