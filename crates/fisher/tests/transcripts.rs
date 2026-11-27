// Replay principal variations as the search crate tests do.
#[path = "../../search/src/tests/support/principal_variation.rs"]
mod principal_variation;
#[path = "support/scripted_transcripts.rs"]
mod scripted_transcripts;

use std::{
    collections::VecDeque,
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

use shakmaty::{
    CastlingMode, Chess, EnPassantMode, Move, Position, PositionError, fen::Fen, uci::UciMove,
};
use wait_timeout::ChildExt;

const LEGAL_BESTMOVE: &str = "bestmove {legal}";
const HANG_TIMEOUT: Duration = Duration::from_secs(30);

#[test]
fn handshake() -> io::Result<()> {
    play_transcript("multipv/handshake")
}

#[test]
fn multipv_settings() -> io::Result<()> {
    play_transcript("multipv/settings")
}

#[test]
fn multipv_option_values() -> io::Result<()> {
    play_transcript("multipv/option-values")
}

#[test]
fn multipv_retained_setting() -> io::Result<()> {
    play_transcript("multipv/retained-setting")
}

#[test]
fn multipv_root_move_count() -> io::Result<()> {
    play_transcript("multipv/root-move-count")
}

#[test]
fn multipv_finished_depths() -> io::Result<()> {
    play_transcript("multipv/finished-depths")
}

#[test]
fn multipv_setoption_during_search() -> io::Result<()> {
    play_transcript("multipv/setoption-during-search")
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
fn position_fen_four_fields() -> io::Result<()> {
    play_transcript("position-fen-four-fields")
}

#[test]
fn position_fen_five_fields() -> io::Result<()> {
    play_transcript("position-fen-five-fields")
}

#[test]
fn kingside_k() -> io::Result<()> {
    play_transcript("castling-letters/kingside-k")
}

#[test]
fn kingside_kk() -> io::Result<()> {
    play_transcript("castling-letters/kingside-kk")
}

#[test]
fn kingside_h() -> io::Result<()> {
    play_transcript("castling-letters/kingside-h")
}

#[test]
fn go_parameters() -> io::Result<()> {
    play_transcript("go-parameters")
}

#[test]
fn go_depth() -> io::Result<()> {
    play_transcript("go-depth")
}

#[test]
fn startpos_depth_two() -> io::Result<()> {
    play_transcript("startpos-depth-2")
}

#[test]
fn go_nodes() -> io::Result<()> {
    play_transcript("go-nodes")
}

#[test]
fn go_movetime() -> io::Result<()> {
    play_transcript("go-movetime")
}

#[test]
fn go_nodes_movetime() -> io::Result<()> {
    play_transcript("go-nodes-movetime")
}

#[test]
fn go_white_clock_exhausted() -> io::Result<()> {
    play_transcript("go-white-clock-exhausted")
}

#[test]
fn go_black_clock_exhausted() -> io::Result<()> {
    play_transcript("go-black-clock-exhausted")
}

#[test]
fn go_healthy_clock() -> io::Result<()> {
    play_transcript("go-healthy-clock")
}

#[test]
fn go_clock_terminal() -> io::Result<()> {
    play_transcript("go-clock-terminal")
}

#[test]
fn quit_while_deepening() -> io::Result<()> {
    play_transcript("quit-while-deepening")
}

#[test]
fn end_of_input_while_deepening() -> io::Result<()> {
    play_transcript("end-of-input-while-deepening")
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
fn whitespace_no_break_space_command() -> io::Result<()> {
    play_transcript("whitespace/no-break-space-command")
}

#[test]
fn whitespace_tab_command() -> io::Result<()> {
    play_transcript("whitespace/tab-command")
}

#[test]
fn whitespace_tab_then_isready() -> io::Result<()> {
    play_transcript("whitespace/tab-then-isready")
}

#[test]
fn whitespace_vertical_tab_command() -> io::Result<()> {
    play_transcript("whitespace/vertical-tab-command")
}

#[test]
fn whitespace_vertical_tab_then_isready() -> io::Result<()> {
    play_transcript("whitespace/vertical-tab-then-isready")
}

#[test]
fn whitespace_no_break_space_in_fen() -> io::Result<()> {
    play_transcript("whitespace/no-break-space-in-fen")
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
fn go_then_quit() -> io::Result<()> {
    play_transcript("go-then-quit")
}

#[test]
fn go_then_end_of_input() -> io::Result<()> {
    play_transcript("go-then-end-of-input")
}

#[test]
fn stop_position_go() -> io::Result<()> {
    play_transcript("stop-position-go")
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
    run_transcript(name, &transcript, HANG_TIMEOUT, Engine::start)
}

fn run_transcript<I: TranscriptIo>(
    name: &str,
    transcript: &str,
    timeout: Duration,
    start: impl FnOnce() -> io::Result<I>,
) -> io::Result<Vec<String>> {
    let records = load_transcript(name, transcript)?;
    Session::new(start()?, timeout).play(name, records)
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

fn command_tokens(text: &str) -> impl Iterator<Item = &str> {
    text.split(|ch: char| ch.is_ascii_whitespace() || ch == '\x0b')
        .filter(|token| !token.is_empty())
}

/// A position as fisher holds it. shakmaty keeps the move counters as `u32`
/// and fisher as `u64`, so the runner keeps fisher's counters itself.
#[derive(Clone)]
struct Tracked {
    chess: Chess,
    halfmove: u64,
    fullmove: u64,
}

impl Tracked {
    const STARTING: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    /// Sets up a six-field FEN by fisher's rules.
    fn from_fen(fen: &str) -> Result<Self, String> {
        let (halfmove, fullmove) =
            check_fen_text(fen).map_err(|error| format!("invalid FEN {fen:?}: {error}"))?;
        // fisher does not reject an impossible check or too much material,
        // so the runner accepts them too.
        let chess = fen
            .parse::<Fen>()
            .map_err(|error| format!("invalid FEN {fen:?}: {error}"))?
            .into_position(CastlingMode::Standard)
            .or_else(PositionError::ignore_too_much_material)
            .or_else(PositionError::ignore_impossible_check)
            .map_err(|error| format!("invalid position {fen:?}: {error}"))?;
        Ok(Self {
            chess,
            halfmove,
            fullmove,
        })
    }

    /// The FEN of the position with fisher's counters, and an en passant
    /// square only when a capture there is legal.
    fn fen(&self) -> String {
        let fen = Fen::from_position(&self.chess, EnPassantMode::Legal).to_string();
        let fields: Vec<_> = fen.split(' ').take(4).collect();
        format!("{} {} {}", fields.join(" "), self.halfmove, self.fullmove)
    }
}

/// Sets up a position the way `uci` does: `startpos`, or a FEN of four, five
/// or six fields, where a missing halfmove clock is 0 and a missing fullmove
/// number is 1, followed by optional moves.
fn reference_position<'a>(mut tokens: impl Iterator<Item = &'a str>) -> Result<Tracked, String> {
    let mut tokens = tokens.by_ref().peekable();
    let mut position = match tokens.next() {
        Some("startpos") => Tracked::from_fen(Tracked::STARTING)?,
        Some("fen") => {
            let mut fields = Vec::new();
            while let Some(field) = tokens.next_if(|&token| token != "moves") {
                fields.push(field);
            }
            Tracked::from_fen(&match fields.len() {
                4 => format!("{} 0 1", fields.join(" ")),
                5 => format!("{} 1", fields.join(" ")),
                _ => fields.join(" "),
            })?
        }
        _ => return Err("expected startpos or fen".to_owned()),
    };
    match tokens.next() {
        None => return Ok(position),
        Some("moves") => {}
        Some(token) => return Err(format!("expected moves, got {token:?}")),
    }
    // The counters change as `rules` changes them.
    for token in tokens {
        let movement = reference_move(&position.chess, token)
            .ok_or_else(|| format!("illegal move {token}"))?;
        position.halfmove = if movement.is_zeroing() {
            0
        } else {
            position.halfmove.saturating_add(1)
        };
        let mover = movement
            .from()
            .and_then(|square| position.chess.board().color_at(square));
        if mover == Some(shakmaty::Color::Black) {
            position.fullmove = position.fullmove.saturating_add(1);
        }
        position.chess.play_unchecked(movement);
    }
    Ok(position)
}

/// Checks the text of a UCI FEN. Apart from the complete castling fields
/// `KK` and `H`, it follows `rules::Position::from_fen`.
/// Shakmaty also reads missing fields,
/// pockets, promoted pieces, check counters, `_` between fields, castling by
/// file, clocks beyond `u32` and a fullmove number of 0. The rules about
/// kings, pawns, castling pieces and the en passant pawn are the same in both
/// and are left to shakmaty. Returns the move counters as fisher reads them.
fn check_fen_text(fen: &str) -> Result<(u64, u64), &'static str> {
    let [pieces, color, castling, en_passant, halfmove, fullmove] =
        command_tokens(fen).collect::<Vec<_>>()[..]
    else {
        return Err("FEN must have six fields");
    };
    let ranks: Vec<_> = pieces.split('/').collect();
    if ranks.len() != 8 {
        return Err("piece placement must have eight ranks");
    }
    for rank in ranks {
        let mut file = 0;
        for byte in rank.bytes() {
            if file >= 8 {
                return Err("each rank must have eight files");
            }
            match byte {
                b'1'..=b'8' => file += byte - b'0',
                b'p' | b'n' | b'b' | b'r' | b'q' | b'k' | b'P' | b'N' | b'B' | b'R' | b'Q'
                | b'K' => file += 1,
                _ => return Err("unknown piece letter in FEN"),
            }
        }
        if file != 8 {
            return Err("each rank must have eight files");
        }
    }
    let en_passant_rank = match color {
        "w" => b'6',
        "b" => b'3',
        _ => return Err("side to move must be w or b"),
    };
    if !matches!(castling, "-" | "KK" | "H") {
        let mut seen = Vec::new();
        for letter in castling.chars() {
            if !"KQkq".contains(letter) {
                return Err("castling rights must use K, Q, k, q, or -");
            }
            if seen.contains(&letter) {
                return Err("castling rights must not contain duplicates");
            }
            seen.push(letter);
        }
    }
    match *en_passant.as_bytes() {
        [b'-'] => {}
        [b'a'..=b'h', rank] if rank == en_passant_rank => {}
        [b'a'..=b'h', b'1'..=b'8'] => {
            return Err("en passant square has the wrong rank for the side to move");
        }
        _ => return Err("invalid en passant square"),
    }
    let decimal = |text: &str| {
        text.bytes()
            .all(|byte| byte.is_ascii_digit())
            .then(|| text.parse::<u64>().ok())
            .flatten()
    };
    let halfmove = decimal(halfmove).ok_or("halfmove clock must be an unsigned decimal integer")?;
    let fullmove = decimal(fullmove)
        .filter(|&number| number > 0)
        .ok_or("fullmove number must be a positive decimal integer")?;
    Ok((halfmove, fullmove))
}

fn reference_move(position: &Chess, text: &str) -> Option<Move> {
    position
        .legal_moves()
        .into_iter()
        .find(|&movement| UciMove::from_standard(movement).to_string() == text)
}

fn reply_matches(expected: &str, actual: &str, position: Option<&Chess>) -> bool {
    match expected {
        "id name fisher {version}" => {
            actual == concat!("id name fisher ", env!("CARGO_PKG_VERSION"))
        }
        LEGAL_BESTMOVE => actual.strip_prefix("bestmove ").is_some_and(|text| {
            position.is_some_and(|position| reference_move(position, text).is_some())
        }),
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

/// The deepest searched line whose principal variation the runner replays,
/// and the deepest `{score}` that must come from a position the tests of
/// `search` compare with plain minimax at depths 1 to 4. Deeper lines get the
/// spelling, length and node-rate checks only, as the `full-spelling` fixture
/// in `uci-runner/` expects.
const REPLAY_DEPTH: u8 = 4;

/// The positions where a transcript leaves the score open. The tests of
/// `search` compare each of them with plain minimax at depths 1 to 4.
const OPEN_SCORES: &str = include_str!("../../../test-data/open-scores.txt");

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
            !reply_matches(expected, actual, Some(&position)),
            "{expected:?} must compare exactly"
        );
    }
    assert!(reply_matches(
        "info depth 4 score {score} nodes {nodes}",
        "info depth 4 score cp 0 nodes 206603",
        Some(&position)
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
            reply_matches(expected, actual, Some(&position)),
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
        ("cp 0 nodes 1 nps 020 time 50 pv b1a3", false),
        ("cp 0 nodes 0 nps 00 time 1 pv a7a8q", false),
        ("cp 0 nodes 20 nps -1 time 1 pv b1a3", false),
        ("cp 0 nodes 20 nps 20000 time 1 pv 0000", false),
        ("cp 0 nodes 20 nps 20000 time 1 pv a7a8K", false),
        ("cp 0 nodes 0 nps 0 time 1 pv a7a8k", false),
        ("cp 0 nodes 0 nps 0 time 1 pv a7a8p", false),
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
    multipv: u8,
    time: &'a str,
    nps: &'a str,
    pv: &'a str,
}

fn search_info(text: &str) -> Option<SearchInfo<'_>> {
    let (depth, rest) = text.strip_prefix("info depth ")?.split_once(" score ")?;
    let (score, rest) = rest.split_once(" nodes ")?;
    let (depth, nodes, full) = if let Some((depth, multipv)) = depth.split_once(" multipv ") {
        let multipv = positive_decimal(multipv).then(|| multipv.parse().ok())??;
        let (nodes, rest) = rest.split_once(" nps ")?;
        let (nps, rest) = rest.split_once(" time ")?;
        let (time, pv) = rest.split_once(" pv ")?;
        (
            depth,
            nodes,
            Some(FullSearchInfo {
                multipv,
                time,
                nps,
                pv,
            }),
        )
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
                    expected.multipv == actual.multipv
                        && slot(expected.time, "{time}", actual.time)
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

fn reported_score(text: &str, depth: u8) -> io::Result<i32> {
    if let Some(cp) = text.strip_prefix("cp ") {
        return cp.parse().map_err(io::Error::other);
    }
    let mate = text
        .strip_prefix("mate ")
        .ok_or_else(|| io::Error::other("missing score"))?
        .parse::<i16>()
        .map(i32::from)
        .map_err(io::Error::other)?;
    if mate == 0 {
        return Err(io::Error::other("mate 0 at a searched depth"));
    }
    let plies = 2 * mate.abs() - i32::from(mate > 0);
    if plies > i32::from(depth) {
        return Err(io::Error::other(format!(
            "mate {mate} exceeds the search depth {depth}"
        )));
    }
    Ok(mate.signum() * (principal_variation::MATE_SCORE - plies))
}

/// A `{score}` at depths 1 to `REPLAY_DEPTH` must come from a position
/// listed in `test-data/open-scores.txt`.
fn verify_open_score_is_covered(position: &Tracked, depth: u8) -> io::Result<()> {
    let fen = position.fen();
    let covered = OPEN_SCORES.lines().any(|listed| listed == fen);
    if covered {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "{{score}} at depth {depth} from {fen} is not compared with plain minimax. \
             Add the position to test-data/open-scores.txt"
        )))
    }
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
    fn wait_for_success(&mut self, timeout: Duration) -> io::Result<()>;
}

struct Engine {
    child: Option<Child>,
    output: Receiver<io::Result<String>>,
}

impl Engine {
    fn start() -> io::Result<Self> {
        let mut child = {
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
        let stdout = child
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

        Ok(Self {
            child: Some(child),
            output,
        })
    }
}

impl TranscriptIo for Engine {
    fn send(&mut self, command: &[u8]) -> io::Result<()> {
        let stdin = self
            .child
            .as_mut()
            .and_then(|child| child.stdin.as_mut())
            .ok_or_else(|| io::Error::other("fisher stdin is closed"))?;
        stdin.write_all(command)?;
        stdin.flush()
    }

    fn close_input(&mut self) {
        if let Some(child) = &mut self.child {
            drop(child.stdin.take());
        }
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

    fn wait_for_success(&mut self, timeout: Duration) -> io::Result<()> {
        let child = self
            .child
            .as_mut()
            .ok_or_else(|| io::Error::other("already waited for fisher to exit"))?;
        // Keep stdin open during the wait unless the transcript closed it.
        let _stdin = child.stdin.take();
        let status = child.wait_timeout(timeout)?;
        if status.is_none() {
            drop(child.kill());
            drop(child.wait());
        }
        self.child = None;
        let status = status.ok_or_else(exit_timeout)?;
        if status.success() {
            Ok(())
        } else {
            Err(io::Error::other(format!("fisher exited with {status}")))
        }
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

/// The runner's model of fisher. It knows a search's position only when the
/// search started while fisher was known to be idle, from a position sent
/// while fisher was known to be idle. It never predicts a race: a command
/// sent while a search might still be running makes the position unknown.
struct Session<I> {
    io: I,
    timeout: Duration,
    /// fisher's position, or `None` when the runner cannot know it.
    position: Option<Tracked>,
    /// Each search whose `bestmove` the runner has not read, oldest first.
    /// Each one was certainly started, so each gets exactly one `bestmove`.
    searches: VecDeque<Search>,
    /// A `go` was sent while a search might still be running. The runner
    /// cannot tell whether fisher started it, so it cannot tell which search
    /// any later output belongs to, for the rest of the session.
    lost: bool,
    /// `quit` was sent: fisher reads nothing after it.
    quit: bool,
    /// The last byte sent was CR. Every record ends a line, so no line spans
    /// two records, but an LF that starts one may end no line.
    previous_cr: bool,
    finished_depth: Option<FinishedDepth>,
    legal_bestmove_replies: Vec<String>,
}

struct Search {
    position: Option<Tracked>,
    infinite: bool,
    /// `stop` was sent: fisher finishes this search before it reads on.
    stopped: bool,
}

/// The searched lines of the last finished depth, in the order they arrived.
struct FinishedDepth {
    depth: u8,
    totals: [String; 3],
    root_moves: Vec<String>,
    last_score: i32,
    consecutive: bool,
}

impl FinishedDepth {
    fn observe(
        &mut self,
        depth: u8,
        info: &FullSearchInfo<'_>,
        totals: [&str; 3],
        root_move: &str,
        score: i32,
    ) -> io::Result<()> {
        if depth != self.depth || totals != self.totals.each_ref().map(String::as_str) {
            return Err(io::Error::other(
                "variations of a finished depth disagree on depth, nodes, time or nps",
            ));
        }
        let expected = self.root_moves.len() + 1;
        if usize::from(info.multipv) != expected {
            return Err(io::Error::other(format!(
                "expected multipv {expected}, got {}",
                info.multipv
            )));
        }
        if self.root_moves.iter().any(|movement| movement == root_move) {
            return Err(io::Error::other(format!(
                "repeated root move {root_move} at depth {depth}"
            )));
        }
        if self.last_score < score {
            return Err(io::Error::other(format!(
                "variations are out of score order at depth {depth}"
            )));
        }
        self.root_moves.push(root_move.to_owned());
        self.last_score = score;
        Ok(())
    }
}

impl<I: TranscriptIo> Session<I> {
    fn new(io: I, timeout: Duration) -> Self {
        Self {
            io,
            timeout,
            position: Tracked::from_fen(Tracked::STARTING).ok(),
            searches: VecDeque::new(),
            lost: false,
            quit: false,
            previous_cr: false,
            finished_depth: None,
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
                self.track_input(&command);
                Ok(())
            }
            Record::CloseInput => {
                self.io.close_input();
                Ok(())
            }
            Record::Observe(pattern, delimiter) => self.observe(pattern.as_ref(), delimiter),
        }
    }

    /// Splits input into lines as `uci` does: CR ends a line, and so does LF
    /// unless it follows CR.
    fn track_input(&mut self, bytes: &[u8]) {
        let mut line = Vec::new();
        for &byte in bytes {
            if byte == b'\r' || (byte == b'\n' && !self.previous_cr) {
                self.track_line(&std::mem::take(&mut line));
            } else if byte != b'\n' {
                line.push(byte);
            }
            self.previous_cr = byte == b'\r';
        }
    }

    /// Follows one command. Two facts about a running search are certain
    /// from `uci`'s session, and the runner uses only those: an infinite
    /// search keeps fisher busy until `stop`, so fisher rejects `position`
    /// and `go` until then; and after `stop`, fisher finishes the search
    /// before it reads on.
    fn track_line(&mut self, line: &[u8]) {
        if self.quit {
            return;
        }
        let text = String::from_utf8_lossy(line);
        let mut tokens = command_tokens(&text);
        let running = self.searches.back().filter(|search| !search.stopped);
        let maybe_running = running.is_some();
        match reference_command(&mut tokens) {
            // fisher rejects these during an infinite search, or does
            // nothing for `ucinewgame`.
            Some("position" | "go" | "ucinewgame")
                if running.is_some_and(|search| search.infinite) => {}
            Some("position" | "ucinewgame") if maybe_running => self.position = None,
            // A position fisher rejects leaves its position in place.
            Some("position") => {
                if let Ok(position) = reference_position(tokens) {
                    self.position = Some(position);
                }
            }
            Some("go") if maybe_running => self.lost = true,
            Some("go") => {
                self.searches.push_back(Search {
                    position: self.position.clone(),
                    infinite: tokens.any(|token| token == "infinite"),
                    stopped: false,
                });
            }
            Some("stop") => {
                if let Some(search) = self.searches.back_mut() {
                    search.stopped = true;
                }
            }
            Some("quit") => self.quit = true,
            _ => {}
        }
    }

    /// The position of the search whose output the runner reads now.
    fn search_position(&self, need: &str) -> io::Result<Tracked> {
        self.searches
            .front()
            .and_then(|search| search.position.clone())
            .filter(|_| !self.lost)
            .ok_or_else(|| {
                io::Error::other(format!(
                    "{need} from a search whose position the runner cannot know: a position \
                     or go was sent while a search might still be running. Read bestmove \
                     before sending it, or use a literal value"
                ))
            })
    }

    fn observe(
        &mut self,
        pattern: Option<&SearchInfo<'_>>,
        delimiter: Delimiter<'_>,
    ) -> io::Result<()> {
        let deadline = self.io.now() + self.timeout;
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
            (Delimiter::Exit, None) => self.io.wait_for_success(self.timeout),
        }
    }

    fn expect_reply(&mut self, expected: &str, actual: String) -> io::Result<()> {
        let is_bestmove = actual.starts_with("bestmove ");
        let position = if expected == LEGAL_BESTMOVE {
            Some(self.search_position("bestmove {legal}")?.chess)
        } else {
            None
        };
        if !reply_matches(expected, &actual, position.as_ref()) {
            return Err(io::Error::other(format!(
                "expected {expected:?}, got {actual:?}"
            )));
        }
        if let Some((pattern, info)) = search_info(expected).zip(search_info(&actual)) {
            self.verify_search_info(pattern.score, &info)?;
        } else if let Some(finished) = &mut self.finished_depth {
            finished.consecutive = false;
        }
        if let Some(bestmove) = actual.strip_prefix("bestmove ")
            && let Some(finished) = self.finished_depth.take()
            && let [first, ..] = finished.root_moves.as_slice()
            && first != bestmove
        {
            return Err(io::Error::other(format!(
                "principal variation starts with {first}, got bestmove {bestmove}"
            )));
        }
        if expected == LEGAL_BESTMOVE {
            self.legal_bestmove_replies.push(actual);
        }
        if is_bestmove {
            self.searches.pop_front();
        }
        Ok(())
    }

    fn verify_search_info(
        &mut self,
        expected_score: &str,
        info: &SearchInfo<'_>,
    ) -> io::Result<()> {
        // The runner does not know the right score: the tests of `search`
        // compare it with plain minimax, so a score left open must be at a
        // position they cover, with or without a principal variation.
        let depth = info.depth.parse::<u8>().map_err(io::Error::other)?;
        let checked = (1..=REPLAY_DEPTH).contains(&depth);
        if expected_score == "{score}" && checked {
            verify_open_score_is_covered(&self.search_position("{score}")?, depth)?;
        }
        let Some(full) = &info.full else {
            self.finished_depth = None;
            return Ok(());
        };
        let score = reported_score(info.score, depth)?;
        // Searched lines at depths 1 to 4 get the replay of D5.
        if checked {
            let position = self.search_position("a principal variation")?;
            principal_variation::verify(&position.chess, depth, score, full.pv.split(' '))
                .map_err(io::Error::other)?;
        }
        self.track_variation(depth, full, [info.nodes, full.time, full.nps], score)
    }

    fn track_variation(
        &mut self,
        depth: u8,
        info: &FullSearchInfo<'_>,
        totals: [&str; 3],
        score: i32,
    ) -> io::Result<()> {
        let root_move = info.pv.split(' ').next().unwrap_or_default();
        if info.multipv == 1 {
            if self
                .finished_depth
                .as_ref()
                .is_some_and(|last| depth <= last.depth)
            {
                return Err(io::Error::other(format!(
                    "multipv 1 did not start a new finished depth at depth {depth}"
                )));
            }
            self.finished_depth = Some(FinishedDepth {
                depth,
                totals: totals.map(str::to_owned),
                root_moves: vec![root_move.to_owned()],
                last_score: score,
                consecutive: true,
            });
            return Ok(());
        }
        let finished = self
            .finished_depth
            .as_mut()
            .filter(|last| last.consecutive)
            .ok_or_else(|| io::Error::other("expected a consecutive variation after multipv 1"))?;
        finished.observe(depth, info, totals, root_move, score)
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        if let Some(child) = &mut self.child {
            drop(child.kill());
            drop(child.wait());
        }
    }
}

/// A stand-in for fisher that answers the nth command with the nth batch of
/// lines, for checking the runner itself.
struct FakeEngine {
    replies: std::collections::VecDeque<Vec<&'static str>>,
    stdout: std::collections::VecDeque<&'static str>,
    started: Instant,
}

impl TranscriptIo for FakeEngine {
    fn send(&mut self, _: &[u8]) -> io::Result<()> {
        self.stdout
            .extend(self.replies.pop_front().unwrap_or_default());
        Ok(())
    }

    fn close_input(&mut self) {}

    fn now(&self) -> Instant {
        self.started
    }

    fn read_reply(&mut self, _: Instant) -> io::Result<Option<String>> {
        Ok(self.stdout.pop_front().map(str::to_owned))
    }

    fn wait_for_success(&mut self, _: Duration) -> io::Result<()> {
        Ok(())
    }
}

fn run_against_fake(transcript: &str, replies: Vec<Vec<&'static str>>) -> io::Result<Vec<String>> {
    run_transcript("fake", transcript, HANG_TIMEOUT, || {
        Ok(FakeEngine {
            replies: replies.into(),
            stdout: std::collections::VecDeque::new(),
            started: Instant::now(),
        })
    })
}

fn rejection(result: io::Result<Vec<String>>) -> String {
    result
        .err()
        .map(|error| error.to_string())
        .unwrap_or_default()
}

#[test]
fn checks_every_variation_and_uses_the_first_for_bestmove() {
    // D5 and the MultiPV section of expectations/README.md.
    let first = "info depth 1 multipv 1 score cp 0 nodes 20 nps 20000 time 1 pv e2e4";
    let second = "info depth 1 multipv 2 score cp 0 nodes 20 nps 20000 time 1 pv d2d4";
    for (depth, multipv, other, bestmove, error) in [
        (1, 2, second, "bestmove e2e4", ""),
        (
            1,
            2,
            second,
            "bestmove d2d4",
            "principal variation starts with e2e4",
        ),
        (
            1,
            2,
            "info depth 1 multipv 2 score cp 0 nodes 20 nps 20000 time 1 pv e2e4",
            "bestmove e2e4",
            "repeated root move",
        ),
        (
            1,
            2,
            "info depth 1 multipv 2 score cp 0 nodes 20 nps 20000 time 1 pv e2e5",
            "bestmove e2e4",
            "illegal principal variation move",
        ),
        (
            1,
            2,
            "info depth 1 multipv 2 score cp 0 nodes 20 nps 10000 time 2 pv d2d4",
            "bestmove e2e4",
            "disagree on depth, nodes, time or nps",
        ),
        (
            1,
            2,
            "info depth 1 multipv 2 score cp 0 nodes 21 nps 21000 time 1 pv d2d4",
            "bestmove e2e4",
            "disagree on depth, nodes, time or nps",
        ),
        (
            2,
            2,
            "info depth 2 multipv 2 score cp 0 nodes 20 nps 20000 time 1 pv d2d4 d7d5",
            "bestmove e2e4",
            "disagree on depth, nodes, time or nps",
        ),
        (
            1,
            3,
            "info depth 1 multipv 3 score cp 0 nodes 20 nps 20000 time 1 pv d2d4",
            "bestmove e2e4",
            "expected multipv 2",
        ),
    ] {
        let transcript = format!(
            "> go depth 1\n< {first}\n\
             < info depth {depth} multipv {multipv} score cp 0 nodes {{nodes}} nps {{nps}} time {{time}} pv {{pv}}\n\
             < bestmove {{legal}}\n! exit\n"
        );
        let result = run_against_fake(&transcript, vec![vec![first, other, bestmove]]);
        if error.is_empty() {
            assert!(result.is_ok(), "{result:?}");
        } else {
            let rejected = rejection(result);
            assert!(rejected.contains(error), "{rejected:?}");
        }
    }
}

#[test]
fn rejects_variations_in_the_wrong_score_order() {
    // D13 ranks the scores. Both starting-position variations pass D5:
    // exd5 wins a pawn, and the line ending Nf3 leaves material equal.
    let transcript = "> go depth 3\n\
                      < info depth 3 multipv 1 score {score} nodes 60 nps 60000 time 1 pv {pv}\n\
                      < info depth 3 multipv 2 score {score} nodes 60 nps 60000 time 1 pv {pv}\n\
                      < bestmove {legal}\n! exit\n";
    let ordered = run_against_fake(
        transcript,
        vec![vec![
            "info depth 3 multipv 1 score cp 100 nodes 60 nps 60000 time 1 pv e2e4 d7d5 e4d5",
            "info depth 3 multipv 2 score cp 0 nodes 60 nps 60000 time 1 pv d2d4 d7d5 g1f3",
            "bestmove e2e4",
        ]],
    );
    assert!(ordered.is_ok(), "{ordered:?}");

    let reversed = run_against_fake(
        transcript,
        vec![vec![
            "info depth 3 multipv 1 score cp 0 nodes 60 nps 60000 time 1 pv d2d4 d7d5 g1f3",
            "info depth 3 multipv 2 score cp 100 nodes 60 nps 60000 time 1 pv e2e4 d7d5 e4d5",
            "bestmove d2d4",
        ]],
    );
    let error = rejection(reversed);
    assert!(
        error.contains("variations are out of score order at depth 3"),
        "{error:?}"
    );
}

// k7/8/8/8/8/8/8/K7 w is legal and missing from test-data/open-scores.txt.
// 7k/8/5K1Q/8/8/8/8/8 b is listed there.

#[test]
fn open_score_without_a_variation_must_be_covered() {
    let transcript = |position: &str| {
        format!(
            "> position {position}\n> go depth 4\n\
             < info depth 4 score {{score}} nodes {{nodes}}\n< bestmove a1a2\n! exit\n"
        )
    };
    let replies = || {
        vec![
            vec![],
            vec!["info depth 4 score cp 0 nodes 99", "bestmove a1a2"],
        ]
    };
    let error = rejection(run_against_fake(
        &transcript("fen k7/8/8/8/8/8/8/K7 w - - 0 1"),
        replies(),
    ));
    assert!(error.contains("not compared with plain minimax"), "{error}");
    assert!(run_against_fake(&transcript("fen 7k/8/5K1Q/8/8/8/8/8 b - - 0 1"), replies()).is_ok());
}

#[test]
fn repeated_open_score_without_a_variation_must_be_covered() {
    let transcript = "> position fen k7/8/8/8/8/8/8/K7 w - - 0 1\n> go depth 2\n\
                      <* info depth {depth} score {score} nodes {nodes}\n\
                      < bestmove a1a2\n! exit\n";
    let replies = vec![
        vec![],
        vec![
            "info depth 1 score cp 0 nodes 3",
            "info depth 2 score cp 0 nodes 12",
            "bestmove a1a2",
        ],
    ];
    let error = rejection(run_against_fake(transcript, replies));
    assert!(error.contains("not compared with plain minimax"), "{error}");
}

#[test]
fn short_fens_are_set_up_as_uci_sets_them_up() {
    // Before the short FEN, the runner tracks the listed starting position.
    let transcript = |fen: &str| {
        format!(
            "> position startpos\n> position fen {fen}\n> go depth 4\n\
             < info depth 4 score {{score}} nodes {{nodes}}\n< bestmove a1a2\n! exit\n"
        )
    };
    let replies = || {
        vec![
            vec![],
            vec![],
            vec!["info depth 4 score cp 0 nodes 99", "bestmove a1a2"],
        ]
    };
    for fen in ["k7/8/8/8/8/8/8/K7 w - -", "k7/8/8/8/8/8/8/K7 w - - 0"] {
        let error = rejection(run_against_fake(&transcript(fen), replies()));
        // The missing halfmove clock is 0 and the missing fullmove number is 1.
        assert!(
            error.contains("from k7/8/8/8/8/8/8/K7 w - - 0 1 is not compared"),
            "{fen}: {error}"
        );
    }
    assert!(run_against_fake(&transcript("7k/8/5K1Q/8/8/8/8/8 b - -"), replies()).is_ok());
}

#[test]
fn a_record_with_a_cr_sends_two_commands() {
    // `uci` ends a line at CR, so this record sends `ucinewgame` and then a
    // position: the starting position with a halfmove clock of 1, which
    // test-data/open-scores.txt does not list.
    let transcript = |clock: &str| {
        format!(
            "> ucinewgame\\rposition fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR \
             w KQkq - {clock} 1\n> go depth 4\n< info depth 4 score {{score}} nodes {{nodes}}\n\
             < bestmove a2a3\n! exit\n"
        )
    };
    let replies = || {
        vec![
            vec![],
            vec!["info depth 4 score cp 0 nodes 99", "bestmove a2a3"],
        ]
    };
    let error = rejection(run_against_fake(&transcript("1"), replies()));
    assert!(
        error.contains("RNBQKBNR w KQkq - 1 1 is not compared"),
        "{error}"
    );
    assert!(run_against_fake(&transcript("0"), replies()).is_ok());
}

#[test]
fn a_stopped_search_reports_on_its_own_position() {
    // After stop, fisher finishes the search before it reads the next
    // position, but the search's last report can still be unread.
    let transcript = |searched: &str, next: &str, bestmove: &str| {
        format!(
            "> position fen {searched}\n> go infinite\n> stop\n> position fen {next}\n\
             < info depth 4 score {{score}} nodes {{nodes}}\n< bestmove {bestmove}\n! exit\n"
        )
    };
    let replies = |bestmove| {
        vec![
            vec![],
            vec![],
            vec!["info depth 4 score cp 0 nodes 9", bestmove],
            vec![],
        ]
    };
    let unlisted = "k7/8/8/8/8/8/8/K7 w - - 0 1";
    let listed = "7k/8/5K1Q/8/8/8/8/8 b - - 0 1";
    let error = rejection(run_against_fake(
        &transcript(unlisted, listed, "a1a2"),
        replies("bestmove a1a2"),
    ));
    assert!(error.contains("from k7/8/8/8/8/8/8/K7 w"), "{error}");
    assert!(
        run_against_fake(
            &transcript(listed, unlisted, "h8g8"),
            replies("bestmove h8g8")
        )
        .is_ok()
    );
}

#[test]
fn the_runner_sets_up_the_fens_fisher_accepts() {
    // Expected verdicts from the rules reader and the UCI FEN handling.
    let board = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
    let cases = [
        (format!("{board} w KQkq - 0 1"), true),
        (format!("{board} w KQkq -"), true),
        (format!("{board} w KQkq - 0"), true),
        (format!("{board} w qkQK - 01 1"), true),
        (format!("{board} w KQkq - 99999999999 1"), true),
        // Too much material and an impossible check: fisher does not check.
        ("k7/8/8/8/8/8/PPPPPPPP/QQQQQQQK w - - 0 1".to_owned(), true),
        ("4k3/8/3N1N2/8/8/8/8/4K3 b - - 0 1".to_owned(), true),
        (format!("{board} w KQkq - 0 0"), false),
        (format!("{board} w KQkq - +1 1"), false),
        // fisher-4h7 accepts only the complete fields KK and H as K.
        (format!("{board} w KK - 0 1"), true),
        (format!("{board} w KKK - 0 1"), false),
        (format!("{board} w HAha - 0 1"), false),
        (format!("{board} W KQkq - 0 1"), false),
        (format!("{board} w KQkq - 0 1 +3+3"), false),
        (format!("{board} w KQkq"), false),
        (format!("{board} w KQkq e3 0 1"), false),
        (
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBN~ w Qkq - 0 1".to_owned(),
            false,
        ),
        (
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR[] w KQkq - 0 1".to_owned(),
            false,
        ),
    ];
    for (fen, accepted) in cases {
        let command = format!("fen {fen}");
        assert_eq!(
            reference_position(command_tokens(&command)).is_ok(),
            accepted,
            "{fen}"
        );
    }
}

#[test]
fn the_runner_uses_stockfish_whitespace() {
    for (line, expected) in [
        ("isready\u{a0}x", None),
        ("isready\tx", Some("x")),
        ("isready\tisready", Some("isready")),
        ("isready\x0bx", Some("x")),
        ("isready\x0bisready", Some("isready")),
    ] {
        let mut tokens = command_tokens(line);
        assert_eq!(
            reference_command(&mut tokens),
            expected.map(|_| "isready"),
            "{line:?}"
        );
        assert_eq!(tokens.next(), expected, "{line:?}");
        assert_eq!(tokens.next(), None, "{line:?}");
    }
}

#[test]
fn multipv_setoption_uses_stockfish_whitespace() -> io::Result<()> {
    // option-values.uci requires a diagnostic and retains the setting
    // when a declared option receives a malformed value.
    for (name, separator, value) in [
        ("space", " ", "2"),
        ("tab", "\t", "2"),
        ("vertical-tab", "\x0b", "2"),
        ("no-break-space", " ", "\u{a0}2"),
    ] {
        let command = format!("setoption name MultiPV value{separator}{value}");
        let mut tokens = command_tokens(&command);
        assert_eq!(reference_command(&mut tokens), Some("setoption"), "{name}");
        assert!(tokens.eq(["name", "MultiPV", "value", value]), "{name}");

        let (diagnostic, second_variation) = if value == "2" {
            (
                "",
                "< info depth 1 multipv 2 score cp 0 nodes {nodes} nps {nps} time {time} pv {pv}\n",
            )
        } else {
            ("< info string {message:MultiPV}\n", "")
        };
        let transcript = format!(
            "> {command}\n> isready\n{diagnostic}< readyok\n> go depth 1\n\
             < info depth 1 multipv 1 score cp 0 nodes {{nodes}} nps {{nps}} time {{time}} pv {{pv}}\n\
             {second_variation}< bestmove {{legal}}\n> quit\n! exit\n"
        );
        run_transcript(name, &transcript, HANG_TIMEOUT, Engine::start)?;
    }
    Ok(())
}

#[test]
fn the_runner_keeps_the_position_after_a_joined_en_passant_field() {
    let mut runner = Session::new(
        FakeEngine {
            replies: VecDeque::new(),
            stdout: VecDeque::new(),
            started: Instant::now(),
        },
        HANG_TIMEOUT,
    );
    for fen in [
        "4k3/8/8/8/8/8/8/4K2R w K -\u{a0}0 1",
        "4k3/8/8/8/8/8/8/4K2R w K -\u{a0}7 42",
    ] {
        runner.track_input(format!("position fen {fen}\n").as_bytes());
        assert_eq!(
            runner.position.as_ref().map(Tracked::fen).as_deref(),
            Some(Tracked::STARTING),
            "{fen:?}"
        );
        assert_eq!(
            check_fen_text(fen),
            Err("FEN must have six fields"),
            "{fen:?}"
        );
    }
}

#[test]
fn the_runner_tracks_kingside_castling_spellings() -> Result<(), String> {
    // Stockfish 19 printed these positions for all three fields in fisher-4h7.
    for field in ["K", "KK", "H"] {
        let bare = format!("fen 4k3/8/8/8/8/8/8/4K2R w {field} - 0 1");
        let position = reference_position(command_tokens(&bare))?;
        assert_eq!(position.fen(), "4k3/8/8/8/8/8/8/4K2R w K - 0 1");
        assert_eq!(position.chess.legal_moves().len(), 15);

        let castle = format!("{bare} moves e1g1");
        let position = reference_position(command_tokens(&castle))?;
        assert_eq!(position.fen(), "4k3/8/8/8/8/8/8/5RK1 b - - 1 1");
    }
    Ok(())
}

#[test]
fn a_position_fisher_rejects_leaves_the_position_in_place() {
    // The last rank of the rejected FEN has seven files.
    let transcript = "> position fen 7k/8/5K1Q/8/8/8/8/8 b - - 0 1\n\
                      > position fen k7/8/8/8/8/8/8/K6 w - - 0 1\n\
                      < info string {message:eight files}\n> go depth 4\n\
                      < info depth 4 score {score} nodes {nodes}\n< bestmove h8g8\n! exit\n";
    let replies = vec![
        vec![],
        vec!["info string position rejected: invalid FEN: eight files"],
        vec!["info depth 4 score mate -1 nodes 99", "bestmove h8g8"],
    ];
    assert!(run_against_fake(transcript, replies).is_ok());
}

#[test]
fn a_command_sent_during_a_finite_search_makes_the_position_unknown() {
    // fisher may have finished depth 1 and accepted the position, or not.
    let transcript = |score: &str| {
        format!(
            "> go depth 1\n> position startpos\n\
             < info depth 1 multipv 1 score cp 0 nodes 20 nps 20000 time 1 pv e2e4\n\
             < bestmove e2e4\n> go depth 4\n< info depth 4 score {score} nodes {{nodes}}\n\
             < bestmove e2e4\n! exit\n"
        )
    };
    let replies = || {
        vec![
            vec![
                "info depth 1 multipv 1 score cp 0 nodes 20 nps 20000 time 1 pv e2e4",
                "bestmove e2e4",
            ],
            vec![],
            vec!["info depth 4 score cp 0 nodes 99", "bestmove e2e4"],
        ]
    };
    let error = rejection(run_against_fake(&transcript("{score}"), replies()));
    assert!(
        error.contains("{score} from a search whose position"),
        "{error}"
    );
    assert!(error.contains("Read bestmove before sending it"), "{error}");
    assert!(run_against_fake(&transcript("cp 0"), replies()).is_ok());
}

#[test]
fn a_go_sent_during_a_finite_search_leaves_every_later_position_unknown() {
    // fisher may have started the second search, or rejected it.
    let transcript = "> go depth 1\n> go depth 1\n< bestmove e2e4\n> position startpos\n\
                      > go depth 4\n< info depth 4 score {score} nodes {nodes}\n\
                      < bestmove e2e4\n! exit\n";
    let replies = vec![
        vec!["bestmove e2e4"],
        vec![],
        vec![],
        vec!["info depth 4 score cp 0 nodes 99", "bestmove e2e4"],
    ];
    let error = rejection(run_against_fake(transcript, replies));
    assert!(
        error.contains("{score} from a search whose position"),
        "{error}"
    );
}

#[test]
fn fisher_rejects_position_and_go_during_an_infinite_search() {
    // Until stop, an infinite search keeps fisher busy, whatever comes next.
    for (next, replies) in [
        (
            "isready\n< info string {message:waiting for stop}\n< readyok",
            vec!["readyok"],
        ),
        (
            "ucinewgame\n< info string {message:waiting for stop}",
            vec![],
        ),
        ("\n< info string {message:waiting for stop}", vec![]),
        (
            "position startpos\\r\\r\n< info string {message:waiting for stop}\n\
             < info string {message:waiting for stop}",
            vec!["info string position rejected: waiting for stop"],
        ),
        (
            "go depth 4\n< info string {message:waiting for stop}\n\
          < info string {message:waiting for stop}",
            vec!["info string go rejected: waiting for stop"],
        ),
    ] {
        let transcript = format!(
            "> go infinite\n> position fen k7/8/8/8/8/8/8/K7 w - - 0 1\n> {next}\n\
             > stop\n< bestmove e2e4\n> go depth 4\n\
             < info depth 4 score {{score}} nodes {{nodes}}\n< bestmove e2e4\n! exit\n"
        );
        let replies = vec![
            vec![],
            vec!["info string position rejected: waiting for stop"],
            replies,
            vec!["bestmove e2e4"],
            vec!["info depth 4 score cp 0 nodes 99", "bestmove e2e4"],
        ];
        // The starting position stays, and test-data/open-scores.txt lists it.
        let result = run_against_fake(&transcript, replies);
        assert!(result.is_ok(), "{next}: {result:?}");
    }
    let eof = "> go infinite\n> position fen k7/8/8/8/8/8/8/K7 w - - 0 1\n\
               < info string {message:waiting for stop}\n! eof\n< bestmove e2e4\n! exit\n";
    let replies = vec![
        vec![],
        vec![
            "info string position rejected: waiting for stop",
            "bestmove e2e4",
        ],
    ];
    assert!(run_against_fake(eof, replies).is_ok());
}

#[test]
fn nothing_after_quit_is_read() {
    let replies = vec![vec![]];
    assert!(run_against_fake("> quit\\rposition rubbish\n! exit\n", replies).is_ok());
}

#[test]
fn move_counters_keep_fisher_s_range() {
    // fisher keeps both counters as u64; shakmaty would write 4294967295.
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 4294967296";
    let position = reference_position(command_tokens(&format!("fen {fen}")));
    assert_eq!(position.map(|position| position.fen()), Ok(fen.to_owned()));
    let error = rejection(run_against_fake(
        &format!(
            "> position fen {fen}\n> go depth 4\n< info depth 4 score {{score}} nodes {{nodes}}\n\
             < bestmove e2e4\n! exit\n"
        ),
        vec![
            vec![],
            vec!["info depth 4 score cp 0 nodes 99", "bestmove e2e4"],
        ],
    ));
    assert!(error.contains("- 0 4294967296 is not compared"), "{error}");
}

#[test]
fn a_rejected_go_loses_attribution_for_the_rest_of_the_session() {
    // fisher rejects the second go, so it never gets a bestmove. Neither stop
    // nor a fresh position restores which search later output belongs to:
    // the last search has the queen on h5, which open-scores.txt does not
    // list.
    let transcript = "> position fen 7k/8/5K1Q/8/8/8/8/8 b - - 0 1\n\
                      > go depth 0\\rgo depth 0\n< info string {message:waiting for stop}\n\
                      < bestmove h8g8\n> stop\n> position fen 7k/8/5K1Q/8/8/8/8/8 b - - 0 1\n\
                      > go depth 0\n< bestmove h8g8\n> stop\n\
                      > position fen 7k/8/5K2/7Q/8/8/8/8 b - - 0 1\n> go depth 1\n\
                      < info depth 1 multipv 1 score {score} nodes 1 nps {nps} time {time} pv h8g8\n\
                      < bestmove {legal}\n> quit\n! exit\n";
    let replies = vec![
        vec![],
        vec!["info string go rejected: waiting for stop", "bestmove h8g8"],
        vec![],
        vec![],
        vec!["bestmove h8g8"],
        vec![],
        vec![],
        vec![
            "info depth 1 multipv 1 score cp -900 nodes 1 nps 1000 time 1 pv h8g8",
            "bestmove h8g8",
        ],
        vec![],
    ];
    let error = rejection(run_against_fake(transcript, replies));
    assert!(error.contains("from a search whose position"), "{error}");
}
