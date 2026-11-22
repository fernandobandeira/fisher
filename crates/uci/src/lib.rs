use std::fmt;

use rules::{Move, Promotion, Square};

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Uci,
    IsReady,
    StartPosition,
    UnsupportedPosition,
    Go,
    Quit,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Input {
    Handled(Command),
    Ignored,
    Unknown,
}

/// Reads one line sent by the GUI. An unknown prefix does not hide a command.
#[must_use]
pub fn parse_command(line: &str) -> Input {
    let mut tokens = line.split_whitespace();
    while let Some(token) = tokens.next() {
        match token {
            "uci" => return Input::Handled(Command::Uci),
            "isready" => return Input::Handled(Command::IsReady),
            "position" => {
                return Input::Handled(
                    if tokens.next() == Some("startpos") && tokens.next().is_none() {
                        Command::StartPosition
                    } else {
                        Command::UnsupportedPosition
                    },
                );
            }
            "go" => return Input::Handled(Command::Go),
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

#[derive(Debug, PartialEq, Eq)]
pub enum Response<'a> {
    IdName(&'a str),
    IdAuthor(&'a str),
    UciOk,
    ReadyOk,
    BestMove(UciMove),
    InfoString(&'a str),
}

#[derive(Debug, PartialEq, Eq)]
pub struct UciMove {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<Promotion>,
}

impl fmt::Display for Response<'_> {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IdName(name) => write!(output, "id name {name}"),
            Self::IdAuthor(author) => write!(output, "id author {author}"),
            Self::UciOk => output.write_str("uciok"),
            Self::ReadyOk => output.write_str("readyok"),
            Self::BestMove(uci_move) => write!(output, "bestmove {uci_move}"),
            Self::InfoString(message) => write!(output, "info string {message}"),
        }
    }
}

impl From<Move> for UciMove {
    fn from(movement: Move) -> Self {
        Self {
            from: movement.from(),
            to: movement.to(),
            promotion: movement.promotion(),
        }
    }
}

impl fmt::Display for UciMove {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(output, "{}{}", self.from, self.to)?;
        match self.promotion {
            Some(Promotion::Knight) => output.write_str("n"),
            Some(Promotion::Bishop) => output.write_str("b"),
            Some(Promotion::Rook) => output.write_str("r"),
            Some(Promotion::Queen) => output.write_str("q"),
            None => Ok(()),
        }
    }
}
