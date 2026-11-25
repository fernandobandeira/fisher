use std::{fmt, iter::Peekable, str::SplitWhitespace};

use crate::UciMove;

const UCI: &str = "uci";
const ISREADY: &str = "isready";
const UCINEWGAME: &str = "ucinewgame";
const POSITION: &str = "position";
const GO: &str = "go";
const STOP: &str = "stop";
const QUIT: &str = "quit";
const SETOPTION: &str = "setoption";
const DEBUG: &str = "debug";
const PONDERHIT: &str = "ponderhit";
const REGISTER: &str = "register";

const STARTPOS: &str = "startpos";
const FEN: &str = "fen";
const MOVES: &str = "moves";

const SEARCHMOVES: &str = "searchmoves";
const PONDER: &str = "ponder";
const INFINITE: &str = "infinite";
const WTIME: &str = "wtime";
const BTIME: &str = "btime";
const WINC: &str = "winc";
const BINC: &str = "binc";
const MOVESTOGO: &str = "movestogo";
const DEPTH: &str = "depth";
const NODES: &str = "nodes";
const MATE: &str = "mate";
const MOVETIME: &str = "movetime";

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Command {
    Uci,
    IsReady,
    UciNewGame,
    Position(Result<PositionCommand, PositionSyntaxError>),
    Go(GoParameters),
    Stop,
    Quit,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Input {
    Handled(Command),
    Ignored,
    Unknown,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct PositionCommand {
    pub(super) source: PositionSource,
    pub(super) moves: Vec<UciMove>,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum PositionSource {
    Startpos,
    Fen(String),
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct PositionSyntaxError(PositionSyntaxCause);

#[derive(Debug, PartialEq, Eq)]
enum PositionSyntaxCause {
    ExpectedSource,
    MissingFen,
    ExpectedMoves,
    InvalidMove(String),
}

impl fmt::Display for PositionSyntaxError {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            PositionSyntaxCause::ExpectedSource => output.write_str("expected startpos or fen"),
            PositionSyntaxCause::MissingFen => output.write_str("expected FEN after fen"),
            PositionSyntaxCause::ExpectedMoves => output.write_str("expected moves after startpos"),
            PositionSyntaxCause::InvalidMove(uci_move) => {
                write!(output, "malformed UCI move {uci_move:?}")
            }
        }
    }
}

impl std::error::Error for PositionSyntaxError {}

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct GoParameters {
    pub(super) searchmoves: Vec<UciMove>,
    pub(super) ponder: bool,
    pub(super) wtime: Option<u64>,
    pub(super) btime: Option<u64>,
    pub(super) winc: Option<u64>,
    pub(super) binc: Option<u64>,
    pub(super) movestogo: Option<u64>,
    pub(super) depth: Option<u64>,
    pub(super) nodes: Option<u64>,
    pub(super) mate: Option<u64>,
    pub(super) movetime: Option<u64>,
    pub(super) infinite: bool,
}

/// Reads one line sent by the GUI. An unknown prefix does not hide a command.
#[must_use]
pub(super) fn parse_command(line: &str) -> Input {
    let mut tokens = line.split_whitespace();
    while let Some(token) = tokens.next() {
        match token {
            UCI => return Input::Handled(Command::Uci),
            ISREADY => return Input::Handled(Command::IsReady),
            UCINEWGAME => return Input::Handled(Command::UciNewGame),
            POSITION => return Input::Handled(Command::Position(parse_position(tokens))),
            GO => return Input::Handled(Command::Go(parse_go(tokens))),
            STOP => return Input::Handled(Command::Stop),
            QUIT => return Input::Handled(Command::Quit),
            // TODO(fisher-a2q): Declare and accept UCI options.
            // TODO(fisher-p25): Handle the debug command.
            // TODO(fisher-2ha): Handle pondering and ponderhit.
            SETOPTION | DEBUG | PONDERHIT | REGISTER => return Input::Ignored,
            _ => {}
        }
    }
    Input::Unknown
}

fn parse_position(tokens: SplitWhitespace<'_>) -> Result<PositionCommand, PositionSyntaxError> {
    let mut tokens = tokens.peekable();
    let source = match tokens.next() {
        Some(STARTPOS) => PositionSource::Startpos,
        Some(FEN) => PositionSource::Fen(read_fen_until_moves(&mut tokens)?),
        _ => return Err(PositionSyntaxError(PositionSyntaxCause::ExpectedSource)),
    };

    let moves = match tokens.next() {
        None => Vec::new(),
        Some(MOVES) => tokens
            .map(|token| {
                UciMove::parse(token).ok_or_else(|| {
                    PositionSyntaxError(PositionSyntaxCause::InvalidMove(token.to_owned()))
                })
            })
            .collect::<Result<_, _>>()?,
        Some(_) => return Err(PositionSyntaxError(PositionSyntaxCause::ExpectedMoves)),
    };
    Ok(PositionCommand { source, moves })
}

fn read_fen_until_moves(
    tokens: &mut Peekable<SplitWhitespace<'_>>,
) -> Result<String, PositionSyntaxError> {
    let mut fields = Vec::new();
    while let Some(field) = tokens.next_if(|token| *token != MOVES) {
        fields.push(field);
    }
    if fields.is_empty() {
        return Err(PositionSyntaxError(PositionSyntaxCause::MissingFen));
    }
    Ok(fields.join(" "))
}

fn parse_go(tokens: SplitWhitespace<'_>) -> GoParameters {
    let mut tokens = tokens.peekable();
    let mut parameters = GoParameters::default();
    while let Some(token) = tokens.next() {
        let limit = match token {
            SEARCHMOVES => {
                while let Some(uci_move) = tokens.peek().and_then(|token| UciMove::parse(token)) {
                    let _consumed = tokens.next();
                    parameters.searchmoves.push(uci_move);
                }
                continue;
            }
            PONDER => {
                parameters.ponder = true;
                continue;
            }
            INFINITE => {
                parameters.infinite = true;
                continue;
            }
            WTIME => &mut parameters.wtime,
            BTIME => &mut parameters.btime,
            WINC => &mut parameters.winc,
            BINC => &mut parameters.binc,
            MOVESTOGO => &mut parameters.movestogo,
            DEPTH => &mut parameters.depth,
            NODES => &mut parameters.nodes,
            MATE => &mut parameters.mate,
            MOVETIME => &mut parameters.movetime,
            _ => continue,
        };
        if let Some(number) = read_number(&mut tokens, matches!(token, WTIME | BTIME)) {
            *limit = Some(number);
        }
    }
    parameters
}

fn read_number(tokens: &mut Peekable<SplitWhitespace<'_>>, is_clock: bool) -> Option<u64> {
    let token = tokens.peek()?;
    let negative_clock = is_clock
        && token.strip_prefix('-').is_some_and(|digits| {
            !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
        });
    let number = if negative_clock {
        0
    } else {
        token.parse().ok()?
    };
    tokens.next()?;
    Some(number)
}

#[cfg(test)]
mod tests {
    mod commands;

    use super::*;

    #[test]
    fn distinguishes_all_position_syntax_errors() {
        for (line, error) in [
            ("position", PositionSyntaxCause::ExpectedSource),
            ("position fen moves e2e4", PositionSyntaxCause::MissingFen),
            ("position startpos e2e4", PositionSyntaxCause::ExpectedMoves),
            (
                "position startpos moves e2e4 e9e5",
                PositionSyntaxCause::InvalidMove("e9e5".to_owned()),
            ),
        ] {
            assert_eq!(
                parse_command(line),
                Input::Handled(Command::Position(Err(PositionSyntaxError(error)))),
                "{line}"
            );
        }
    }
}
