use std::fmt;

use rules::{Move, Promotion, Square};

mod command;

pub use command::{
    Command, GoParameters, Input, PositionCommand, PositionSource, PositionSyntaxError,
    parse_command,
};

#[derive(Debug, PartialEq, Eq)]
pub enum Response<'a> {
    IdName(&'a str),
    IdAuthor(&'a str),
    UciOk,
    ReadyOk,
    BestMove(UciMove),
    InfoString(&'a str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UciMove {
    Normal {
        from: Square,
        to: Square,
        promotion: Option<Promotion>,
    },
    Null,
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
        Self::Normal {
            from: movement.from(),
            to: movement.to(),
            promotion: movement.promotion(),
        }
    }
}

impl fmt::Display for UciMove {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self::Normal {
            from,
            to,
            promotion,
        } = self
        else {
            return output.write_str("0000");
        };
        write!(output, "{from}{to}")?;
        match promotion {
            Some(promotion) => output.write_str(promotion_suffix(*promotion)),
            None => Ok(()),
        }
    }
}

impl UciMove {
    fn parse(text: &str) -> Option<Self> {
        if text == "0000" {
            return Some(Self::Null);
        }
        let from = text.get(..2).and_then(Square::parse)?;
        let to = text.get(2..4).and_then(Square::parse)?;
        let promotion = match text.get(4..)? {
            "" => None,
            suffix => Some(
                [
                    Promotion::Knight,
                    Promotion::Bishop,
                    Promotion::Rook,
                    Promotion::Queen,
                ]
                .into_iter()
                .find(|&promotion| promotion_suffix(promotion) == suffix)?,
            ),
        };
        Some(Self::Normal {
            from,
            to,
            promotion,
        })
    }
}

fn promotion_suffix(promotion: Promotion) -> &'static str {
    match promotion {
        Promotion::Knight => "n",
        Promotion::Bishop => "b",
        Promotion::Rook => "r",
        Promotion::Queen => "q",
    }
}
