use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Uci,
    IsReady,
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
    for token in line.split_whitespace() {
        match token {
            "uci" => return Input::Handled(Command::Uci),
            "isready" => return Input::Handled(Command::IsReady),
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
}

impl fmt::Display for Response<'_> {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IdName(name) => write!(output, "id name {name}"),
            Self::IdAuthor(author) => write!(output, "id author {author}"),
            Self::UciOk => output.write_str("uciok"),
            Self::ReadyOk => output.write_str("readyok"),
        }
    }
}
