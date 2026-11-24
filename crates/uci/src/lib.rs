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
    SearchInfo {
        depth: u8,
        score: SearchScore,
        nodes: Option<u64>,
        time_ms: Option<u64>,
        nps: Option<u64>,
        pv: &'a [UciMove],
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchScore {
    Centipawns(i32),
    Mate { plies: u8, winning: bool },
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
            Self::SearchInfo {
                depth,
                score,
                nodes,
                time_ms,
                nps,
                pv,
            } => {
                write!(output, "info depth {depth}")?;
                if *depth > 0 {
                    output.write_str(" multipv 1")?;
                }
                output.write_str(" score ")?;
                match score {
                    SearchScore::Centipawns(value) => write!(output, "cp {value}")?,
                    SearchScore::Mate { plies, winning } => {
                        let moves = i32::from(plies.div_ceil(2));
                        let moves = if *winning { moves } else { -moves };
                        write!(output, "mate {moves}")?;
                    }
                }
                if let Some(nodes) = nodes {
                    write!(output, " nodes {nodes}")?;
                }
                if let Some(nps) = nps {
                    write!(output, " nps {nps}")?;
                }
                if let Some(time_ms) = time_ms {
                    write!(output, " time {time_ms}")?;
                }
                if !pv.is_empty() {
                    output.write_str(" pv")?;
                    for movement in *pv {
                        write!(output, " {movement}")?;
                    }
                }
                Ok(())
            }
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

#[cfg(test)]
mod tests {
    use super::{Response, SearchScore};

    #[test]
    fn search_info_reports_mate_in_signed_moves() {
        for (plies, winning, expected) in [
            (1, true, "info depth 4 multipv 1 score mate 1 nodes 20"),
            (2, false, "info depth 4 multipv 1 score mate -1 nodes 20"),
            (3, true, "info depth 4 multipv 1 score mate 2 nodes 20"),
        ] {
            let response = Response::SearchInfo {
                depth: 4,
                score: SearchScore::Mate { plies, winning },
                nodes: Some(20),
                time_ms: None,
                nps: None,
                pv: &[],
            };
            assert_eq!(response.to_string(), expected);
        }
    }

    #[test]
    fn terminal_search_info_reports_mate_zero() {
        let response = Response::SearchInfo {
            depth: 0,
            score: SearchScore::Mate {
                plies: 0,
                winning: false,
            },
            nodes: None,
            time_ms: None,
            nps: None,
            pv: &[],
        };
        assert_eq!(response.to_string(), "info depth 0 score mate 0");
    }
}
