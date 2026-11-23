#[path = "support/reference_minimax.rs"]
mod reference;

use std::error::Error;

use rules::Position;
use search::{Score, minimax};
use shakmaty::{Chess, EnPassantMode, Position as _, fen::Fen};
use uci::UciMove;

const PERFT: &str = include_str!("../../../expectations/perft.epd");
const MATE_IN_TWO: &str = include_str!("../../../expectations/mate-in-two.tsv");

fn records(data: &str) -> impl Iterator<Item = &str> {
    data.lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
}

fn check_perft(depth: u8) -> Result<(), Box<dyn Error>> {
    for record in records(PERFT) {
        let (fen, counts) = record.split_once(';').ok_or("missing perft counts")?;
        let mut expected = 0;
        for ply in 1..=depth {
            let label = format!("D{ply} ");
            let count = counts
                .split(';')
                .find_map(|entry| entry.trim().strip_prefix(&label))
                .ok_or_else(|| format!("missing perft count at ply {ply} for {fen}"))?;
            expected += count.parse::<u64>()?;
        }
        let (_, _, nodes) = minimax(&Position::from_fen(fen.trim())?, depth);
        assert_eq!(nodes, expected, "{fen} at depth {depth}");
    }
    Ok(())
}

#[test]
fn node_count_matches_published_perft_at_two_plies() -> Result<(), Box<dyn Error>> {
    check_perft(2)
}

#[test]
#[ignore = "run in release mode for the published four-ply counts"]
fn node_count_matches_published_perft_at_four_plies() -> Result<(), Box<dyn Error>> {
    check_perft(4)
}

fn check_mate_in_two(records: impl Iterator<Item = &'static str>) -> Result<(), Box<dyn Error>> {
    for record in records {
        let (id, fields) = record.split_once('\t').ok_or("missing puzzle ID")?;
        let (fen, expected) = fields.split_once('\t').ok_or("missing published move")?;
        let (chosen, score, _) = minimax(&Position::from_fen(fen)?, 4);
        assert_eq!(
            score,
            Score::Mate {
                plies: 3,
                winning: true
            },
            "{id}"
        );
        assert_eq!(
            chosen
                .map(|movement| UciMove::from(movement).to_string())
                .as_deref(),
            Some(expected),
            "{id}"
        );
    }
    Ok(())
}

#[test]
fn first_published_mate_in_two() -> Result<(), Box<dyn Error>> {
    check_mate_in_two(records(MATE_IN_TWO).take(1))
}

#[test]
#[ignore = "run in release mode for every published mate-in-two position"]
fn all_published_mates_in_two() -> Result<(), Box<dyn Error>> {
    check_mate_in_two(records(MATE_IN_TWO))
}

#[test]
fn score_and_nodes_match_reference_on_random_reachable_positions() -> Result<(), Box<dyn Error>> {
    for mut seed in 1_u64..=3 {
        let mut position = Chess::default();
        for _ in 0..12 {
            let moves = position.legal_moves();
            if moves.is_empty() {
                break;
            }
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            let index = usize::try_from((seed >> 32) % u64::try_from(moves.len())?)?;
            let chosen = moves.get(index).copied().ok_or("missing reference move")?;
            position = position.play(chosen)?;

            let fen = Fen::from_position(&position, EnPassantMode::Always).to_string();
            let (_, expected_score, expected_nodes) = reference::minimax(&position, 2);
            let (_, score, nodes) = minimax(&Position::from_fen(&fen)?, 2);
            let score = match score {
                Score::Centipawns(value) => value,
                Score::Mate { plies, winning } => {
                    let value = 100_000 - i32::from(plies);
                    if winning { value } else { -value }
                }
            };
            assert_eq!((score, nodes), (expected_score, expected_nodes), "{fen}");
        }
    }
    Ok(())
}
