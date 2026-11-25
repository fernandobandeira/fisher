#![cfg(test)]
//! Compares the search that `search` exports with plain minimax, on the
//! corpus of fisher-4yn. Plain minimax supplies every expected value here.

use std::error::Error;

use rules::{Move, Position, Promotion};
use shakmaty::{CastlingMode, Chess, fen::Fen};

use super::{
    mate_in_two,
    perft::records,
    plain_minimax::plain_minimax,
    principal_variation::{self, MATE_SCORE},
};
use crate::{Score, SearchResult, minimax, minimax_until_interrupted};

type TestResult = Result<(), Box<dyn Error>>;

/// Checks the exported search against plain minimax at one depth: the score,
/// the chosen root move, no chosen move at a root with no legal move, the
/// bounds on the node count, and the principal variation under D5.
fn assert_same_as_plain_minimax(
    position: &Position,
    depth: u8,
) -> Result<SearchResult, Box<dyn Error>> {
    let fen = position.to_fen();
    let case = format!("{fen} at depth {depth}");
    let expected = plain_minimax(position, depth);
    let result = minimax(position, depth);

    assert_eq!(result.score, expected.score, "score of {case}");
    assert_eq!(
        result.best_move.map(move_text),
        expected.chosen_root_move().map(move_text),
        "chosen root move of {case}"
    );
    if expected.root_moves.is_empty() {
        assert_eq!(
            result.principal_variation,
            [],
            "principal variation of {case}"
        );
    }
    let root_moves = u64::try_from(expected.root_moves.len())?;
    assert!(
        (root_moves..=expected.nodes).contains(&result.nodes),
        "nodes of {case}: {} is outside {root_moves} to {}",
        result.nodes,
        expected.nodes
    );
    assert_eq!(
        minimax_until_interrupted(position, depth, || false).as_ref(),
        Some(&result),
        "the search that can be interrupted differs on {case}"
    );
    assert_eq!(
        result.principal_variation.first().copied(),
        result.best_move,
        "first move of the principal variation of {case}"
    );
    principal_variation::verify(
        &fen.parse::<Fen>()?
            .into_position::<Chess>(CastlingMode::Standard)?,
        depth,
        score_value(result.score),
        result.principal_variation.iter().copied().map(move_text),
    )?;
    Ok(result)
}

fn score_value(score: Score) -> i32 {
    match score {
        Score::Centipawns(value) => value,
        Score::Mate { plies, winning } => {
            let value = MATE_SCORE - i32::from(plies);
            if winning { value } else { -value }
        }
    }
}

fn move_text(movement: Move) -> String {
    let promotion = match movement.promotion() {
        Some(Promotion::Knight) => "n",
        Some(Promotion::Bishop) => "b",
        Some(Promotion::Rook) => "r",
        Some(Promotion::Queen) => "q",
        None => "",
    };
    format!("{}{}{promotion}", movement.from(), movement.to())
}

fn compare_at_depths(fen: &str, depths: impl IntoIterator<Item = u8>) -> TestResult {
    let position = Position::from_fen(fen)?;
    for depth in depths {
        assert_same_as_plain_minimax(&position, depth)?;
    }
    Ok(())
}

/// Every position of a walk of twelve plies from the starting position, each
/// move picked by a seeded generator from fisher's own legal moves.
fn seeded_walk(mut seed: u64) -> Result<Vec<Position>, Box<dyn Error>> {
    let mut position = Position::starting();
    let mut walk = Vec::new();
    for _ in 0..12 {
        let moves = position.legal_moves();
        if moves.is_empty() {
            break;
        }
        seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        let index = usize::try_from((seed >> 32) % u64::try_from(moves.len())?)?;
        position = moves.get(index).ok_or("missing legal move")?.play();
        walk.push(position);
    }
    Ok(walk)
}

/// The positions of the examples reasoned by hand in `tests/search.rs`.
const EXAMPLES: [&str; 8] = [
    // A free capture.
    "7k/q7/8/8/8/8/8/R6K w - - 0 1",
    // A free capture with a reply that captures nothing.
    "7k/n7/6r1/8/8/8/8/RNBQ3K w - - 0 1",
    // A capture that allows a recapture.
    "7k/8/2p5/3p4/8/8/8/3Q3K w - - 0 1",
    // A capture that stalemates.
    "k7/8/1p6/1Q6/8/8/8/7K w - - 0 1",
    // Checkmated roots.
    "k7/1Q6/2K5/8/8/8/8/8 b - - 0 1",
    "7k/6Q1/6K1/8/8/8/8/8 b - - 0 1",
    // A stalemated root.
    "7k/5Q2/6K1/8/8/8/8/8 b - - 0 1",
    // Mate in one.
    "k7/8/1QK5/8/8/8/8/8 w - - 0 1",
];

#[test]
fn perft_positions_at_depths_one_to_three() -> TestResult {
    for record in records()? {
        compare_at_depths(record.fen, 1..=3)?;
    }
    Ok(())
}

#[test]
fn seeded_walks_at_depths_one_and_two() -> TestResult {
    for seed in 1..=3 {
        for position in seeded_walk(seed)? {
            for depth in 1..=2 {
                assert_same_as_plain_minimax(&position, depth)?;
            }
        }
    }
    Ok(())
}

#[test]
fn examples_at_depths_one_to_three() -> TestResult {
    for fen in EXAMPLES {
        compare_at_depths(fen, 1..=3)?;
    }
    Ok(())
}

#[test]
fn published_mate_in_two_at_depth_four() -> TestResult {
    let records = mate_in_two::records()?;
    let record = records
        .iter()
        .find(|record| record.id == "000Zo")
        .ok_or("missing puzzle 000Zo")?;
    compare_at_depths(record.fen, [4])
}

#[test]
fn published_mates_in_two_on_the_horizon() -> TestResult {
    // The checkmate is three plies away, so at depth 3 it stands on the
    // horizon and is scored by material alone.
    for record in mate_in_two::records()?.into_iter().take(10) {
        compare_at_depths(record.fen, [3])?;
    }
    Ok(())
}

#[test]
#[ignore = "run in release mode for the deeper comparison"]
fn perft_positions_at_depths_four_and_five() -> TestResult {
    for record in records()? {
        compare_at_depths(record.fen, 4..=5)?;
    }
    Ok(())
}

#[test]
#[ignore = "run in release mode for the deeper comparison"]
fn last_position_of_each_seeded_walk_at_depth_four() -> TestResult {
    for seed in 1..=3 {
        let walk = seeded_walk(seed)?;
        let last = walk.last().ok_or("empty walk")?;
        assert_same_as_plain_minimax(last, 4)?;
    }
    Ok(())
}

#[test]
#[ignore = "run in release mode for every published mate-in-two position"]
fn all_published_mates_in_two_at_depth_four() -> TestResult {
    let records = mate_in_two::records()?;
    // The published table is documented as containing 100 positions.
    assert_eq!(records.len(), 100);
    for record in records {
        let result = assert_same_as_plain_minimax(&Position::from_fen(record.fen)?, 4)?;
        assert_eq!(
            result.best_move.map(move_text).as_deref(),
            Some(record.published_move),
            "{}",
            record.id
        );
    }
    Ok(())
}
