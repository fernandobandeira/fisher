use std::{iter::Peekable, str::SplitWhitespace};

use crate::UciMove;

const MOVES: &str = "moves";

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Uci,
    IsReady,
    UciNewGame,
    Position(Result<PositionCommand, PositionSyntaxError>),
    Go(GoParameters),
    Quit,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Input {
    Handled(Command),
    Ignored,
    Unknown,
}

#[derive(Debug, PartialEq, Eq)]
pub struct PositionCommand {
    pub source: PositionSource,
    pub moves: Vec<UciMove>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PositionSource {
    Startpos,
    Fen(String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct PositionSyntaxError(PositionSyntaxCause);

#[derive(Debug, PartialEq, Eq)]
enum PositionSyntaxCause {
    ExpectedSource,
    MissingFen,
    ExpectedMoves,
    InvalidMove(String),
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct GoParameters {
    pub searchmoves: Vec<UciMove>,
    pub ponder: bool,
    pub wtime: Option<u64>,
    pub btime: Option<u64>,
    pub winc: Option<u64>,
    pub binc: Option<u64>,
    pub movestogo: Option<u64>,
    pub depth: Option<u64>,
    pub nodes: Option<u64>,
    pub mate: Option<u64>,
    pub movetime: Option<u64>,
    pub infinite: bool,
}

/// Reads one line sent by the GUI. An unknown prefix does not hide a command.
#[must_use]
pub fn parse_command(line: &str) -> Input {
    let mut tokens = line.split_whitespace();
    while let Some(token) = tokens.next() {
        match token {
            "uci" => return Input::Handled(Command::Uci),
            "isready" => return Input::Handled(Command::IsReady),
            "ucinewgame" => return Input::Handled(Command::UciNewGame),
            "position" => return Input::Handled(Command::Position(parse_position(tokens))),
            "go" => return Input::Handled(Command::Go(parse_go(tokens))),
            "quit" => return Input::Handled(Command::Quit),
            // TODO(fisher-a2q): Declare and accept UCI options.
            // TODO(fisher-p25): Handle the debug command.
            // TODO(fisher-2ha): Handle pondering and ponderhit.
            "setoption" | "debug" | "ponderhit" | "register" => return Input::Ignored,
            _ => {}
        }
    }
    Input::Unknown
}

fn parse_position(tokens: SplitWhitespace<'_>) -> Result<PositionCommand, PositionSyntaxError> {
    let mut tokens = tokens.peekable();
    let source = match tokens.next() {
        Some("startpos") => PositionSource::Startpos,
        Some("fen") => PositionSource::Fen(read_fen_until_moves(&mut tokens)?),
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
            "searchmoves" => {
                while let Some(uci_move) = tokens.peek().and_then(|token| UciMove::parse(token)) {
                    let _consumed = tokens.next();
                    parameters.searchmoves.push(uci_move);
                }
                continue;
            }
            "ponder" => {
                parameters.ponder = true;
                continue;
            }
            "infinite" => {
                parameters.infinite = true;
                continue;
            }
            "wtime" => &mut parameters.wtime,
            "btime" => &mut parameters.btime,
            "winc" => &mut parameters.winc,
            "binc" => &mut parameters.binc,
            "movestogo" => &mut parameters.movestogo,
            "depth" => &mut parameters.depth,
            "nodes" => &mut parameters.nodes,
            "mate" => &mut parameters.mate,
            "movetime" => &mut parameters.movetime,
            _ => continue,
        };
        if let Some(number) = read_number(&mut tokens) {
            *limit = Some(number);
        }
    }
    parameters
}

fn read_number(tokens: &mut Peekable<SplitWhitespace<'_>>) -> Option<u64> {
    let number = tokens.peek()?.parse().ok()?;
    tokens.next()?;
    Some(number)
}

#[cfg(test)]
mod tests {
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
