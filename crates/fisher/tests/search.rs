#[path = "support/reference_minimax.rs"]
mod reference;
mod support;

use std::error::Error;

use rules::Position;
use search::{Score, minimax};
use shakmaty::{Chess, EnPassantMode, Position as _, fen::Fen};
use support::{check_perft, reference_position};
use uci::UciMove;

const MATE_IN_TWO: &str = include_str!("../../../expectations/mate-in-two.tsv");

fn records(data: &str) -> impl Iterator<Item = &str> {
    data.lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
}

fn search_nodes(fen: &str, depth: u8) -> Result<u64, Box<dyn Error>> {
    Ok(minimax(&Position::from_fen(fen)?, depth).nodes)
}

#[test]
fn node_count_matches_published_perft_at_two_plies() -> Result<(), Box<dyn Error>> {
    check_perft(2, search_nodes)
}

#[test]
#[ignore = "run in release mode for the published four-ply counts"]
fn node_count_matches_published_perft_at_four_plies() -> Result<(), Box<dyn Error>> {
    check_perft(4, search_nodes)
}

fn check_mate_in_two(records: impl Iterator<Item = &'static str>) -> Result<(), Box<dyn Error>> {
    for record in records {
        let (id, fields) = record.split_once('\t').ok_or("missing puzzle ID")?;
        let (fen, expected) = fields.split_once('\t').ok_or("missing published move")?;
        let result = minimax(&Position::from_fen(fen)?, 4);
        assert_eq!(
            result.score,
            Score::Mate {
                plies: 3,
                winning: true
            },
            "{id}"
        );
        assert_eq!(
            result
                .chosen_move
                .map(|movement| UciMove::from(movement).to_string())
                .as_deref(),
            Some(expected),
            "{id}"
        );
    }
    Ok(())
}

#[test]
fn first_ten_published_mates_in_two() -> Result<(), Box<dyn Error>> {
    check_mate_in_two(records(MATE_IN_TWO).take(10))
}

#[test]
#[ignore = "run in release mode for every published mate-in-two position"]
fn all_published_mates_in_two() -> Result<(), Box<dyn Error>> {
    check_mate_in_two(records(MATE_IN_TWO))
}

#[test]
fn mate_on_the_horizon_scores_as_material() -> Result<(), Box<dyn Error>> {
    for record in records(MATE_IN_TWO).take(10) {
        let (_, fields) = record.split_once('\t').ok_or("missing puzzle ID")?;
        let (fen, _) = fields.split_once('\t').ok_or("missing published move")?;
        let position = reference_position(fen)?;
        assert_matches_reference(&position, 3)?;
    }
    Ok(())
}

#[test]
fn stalemate_on_the_horizon_scores_as_material() -> Result<(), Box<dyn Error>> {
    let fen = "k7/8/1p6/1Q6/8/8/8/7K w - - 0 1";
    let position: Chess = reference_position(fen)?;
    let chosen = reference::minimax(&position, 1).chosen_move;
    let chosen = chosen.ok_or("reference found no legal move")?;
    assert!(position.clone().play(chosen)?.is_stalemate());
    assert_matches_reference(&position, 1)
}

#[test]
fn equal_scores_keep_the_first_legal_move() {
    let position = Position::starting();
    // None of the starting position's 20 legal moves changes the material.
    let first = position.legal_moves().into_iter().next();
    let result = minimax(&position, 1);
    assert_eq!(result.chosen_move, first);
    assert_eq!(result.score, Score::Centipawns(0));
    assert_eq!(result.nodes, 20);
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

            assert_matches_reference(&position, 2)?;
        }
    }
    Ok(())
}

fn assert_matches_reference(position: &Chess, depth: u8) -> Result<(), Box<dyn Error>> {
    let expected = reference::minimax(position, depth);
    let fen = Fen::from_position(position, EnPassantMode::Always).to_string();
    let result = minimax(&Position::from_fen(&fen)?, depth);
    let score = match result.score {
        Score::Centipawns(value) => value,
        Score::Mate { plies, winning } => {
            let value = reference::MATE_SCORE - i32::from(plies);
            if winning { value } else { -value }
        }
    };
    assert_eq!(score, expected.score, "{fen}");
    assert_eq!(result.nodes, expected.nodes, "{fen}");
    Ok(())
}
