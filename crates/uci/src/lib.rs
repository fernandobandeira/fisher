use std::fmt;

use rules::{Move, Promotion, Square};

mod command;
mod session;

pub use session::run_session;

#[derive(Debug, PartialEq, Eq)]
enum Response<'a> {
    IdName(&'a str),
    IdAuthor(&'a str),
    MultiPvOption,
    UciOk,
    ReadyOk,
    BestMove(UciMove),
    InfoString(&'a str),
    /// A searched `info` line in Stockfish's spelling: depth, `multipv`,
    /// score, nodes, nps, time and pv, each optional field written only when
    /// present. A depth of 0 is the response for a root with no legal move and
    /// carries no `multipv`.
    SearchInfo {
        depth: u8,
        multipv: Option<usize>,
        score: SearchScore,
        nodes: Option<u64>,
        time_ms: Option<u64>,
        nps: Option<u64>,
        pv: &'a [UciMove],
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SearchScore {
    Centipawns(i32),
    Mate {
        plies: u8,
        /// True when the root's side to move gives the checkmate.
        winning: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UciMove {
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
            Self::MultiPvOption => write!(
                output,
                "option name {} type spin default 1 min 1 max 255",
                command::MULTIPV
            ),
            Self::UciOk => output.write_str("uciok"),
            Self::ReadyOk => output.write_str("readyok"),
            Self::BestMove(uci_move) => write!(output, "bestmove {uci_move}"),
            Self::InfoString(message) => write!(output, "info string {message}"),
            Self::SearchInfo {
                depth,
                multipv,
                score,
                nodes,
                time_ms,
                nps,
                pv,
            } => {
                write!(output, "info depth {depth}")?;
                if let Some(multipv) = multipv {
                    write!(output, " multipv {multipv}")?;
                }
                output.write_str(" score ")?;
                write!(output, "{score}")?;
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

impl From<deepening::Score> for SearchScore {
    fn from(score: deepening::Score) -> Self {
        match score {
            deepening::Score::Centipawns(value) => Self::Centipawns(value),
            deepening::Score::Mate { plies, winning } => Self::Mate { plies, winning },
        }
    }
}

impl fmt::Display for SearchScore {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Centipawns(value) => write!(output, "cp {value}"),
            Self::Mate { plies, winning } => {
                let moves = i32::from(plies.div_ceil(2));
                let moves = if *winning { moves } else { -moves };
                write!(output, "mate {moves}")
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
    use super::{Response, SearchScore, UciMove};

    #[test]
    fn search_info_reports_mate_in_signed_moves() {
        for (plies, winning, expected) in [
            (1, true, "info depth 4 multipv 1 score mate 1 nodes 20"),
            (2, false, "info depth 4 multipv 1 score mate -1 nodes 20"),
            (3, true, "info depth 4 multipv 1 score mate 2 nodes 20"),
        ] {
            let response = Response::SearchInfo {
                depth: 4,
                multipv: Some(1),
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
    fn search_info_spells_every_field_in_stockfish_order() {
        let pv = [
            UciMove::parse("e2e4").unwrap(),
            UciMove::parse("e7e5").unwrap(),
        ];
        let response = Response::SearchInfo {
            depth: 2,
            multipv: Some(1),
            score: SearchScore::Centipawns(-15),
            nodes: Some(420),
            time_ms: Some(3),
            nps: Some(140_000),
            pv: &pv,
        };
        assert_eq!(
            response.to_string(),
            "info depth 2 multipv 1 score cp -15 nodes 420 nps 140000 time 3 pv e2e4 e7e5"
        );
    }

    #[test]
    fn terminal_search_info_reports_mate_zero() {
        let response = Response::SearchInfo {
            depth: 0,
            multipv: None,
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
    use rules::Position;
    use shakmaty::{Chess, Position as _, uci::UciMove as ReferenceMove};

    #[test]
    fn promotion_uci_moves_match_shakmaty() {
        for text in ["a7a8n", "a7a8b", "a7a8r", "a7a8q"] {
            let expected = text.parse::<ReferenceMove>().unwrap();
            assert_eq!(
                UciMove::parse(text).unwrap().to_string(),
                expected.to_string()
            );
        }
        assert_eq!(
            UciMove::parse("a7a8x").is_none(),
            "a7a8x".parse::<ReferenceMove>().is_err()
        );
    }

    #[test]
    fn starting_position_has_every_legal_uci_move() {
        let mut expected: Vec<_> = Chess::default()
            .legal_moves()
            .into_iter()
            .map(|movement| ReferenceMove::from_standard(movement).to_string())
            .collect();
        let mut actual: Vec<_> = Position::starting()
            .legal_moves()
            .into_iter()
            .map(|movement| UciMove::from(movement).to_string())
            .collect();
        expected.sort_unstable();
        actual.sort_unstable();

        assert_eq!(actual, expected);
    }
}
