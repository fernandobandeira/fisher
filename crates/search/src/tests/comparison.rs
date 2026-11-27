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
/// bounds on the node count, the principal variation under D5, and the score
/// of every prefix of the principal variation.
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
    check_each_prefix(position, depth, result.score, &result.principal_variation)?;
    Ok(result)
}

/// Plays the principal variation one move at a time. After each move, plain
/// minimax searches what is left of the depth, and its score, seen from the
/// root, must equal the score the search reported. A variation that reaches
/// the score through a reply worse than the best one fails here.
pub(super) fn check_each_prefix(
    root: &Position,
    depth: u8,
    score: Score,
    variation: &[Move],
) -> Result<(), String> {
    let mut position = *root;
    for (ply, &movement) in (1_u8..).zip(variation) {
        let text = move_text(movement);
        if !position.legal_moves().contains(&movement) {
            return Err(format!(
                "{}: {text} is not legal after {} plies",
                root.to_fen(),
                ply - 1
            ));
        }
        position = movement.play();
        let remaining = depth.checked_sub(ply).ok_or_else(|| {
            format!(
                "{}: the principal variation exceeds depth {depth}",
                root.to_fen()
            )
        })?;
        let from_root = seen_from_root(plain_minimax(&position, remaining).score, ply);
        if from_root != score {
            return Err(format!(
                "{} at depth {depth}: after {text}, {ply} plies into the principal variation, \
                 plain minimax scores {from_root:?}, not {score:?}",
                root.to_fen()
            ));
        }
    }
    Ok(())
}

/// Turns a score from the side to move `ply` plies after the root into a score
/// from the root's color, counting those plies in the distance to checkmate.
fn seen_from_root(score: Score, ply: u8) -> Score {
    let same_color = ply.is_multiple_of(2);
    match score {
        Score::Centipawns(value) => Score::Centipawns(if same_color { value } else { -value }),
        Score::Mate { plies, winning } => Score::Mate {
            plies: plies.saturating_add(ply),
            winning: winning == same_color,
        },
    }
}

pub(super) fn score_value(score: Score) -> i32 {
    match score {
        Score::Centipawns(value) => value,
        Score::Mate { plies, winning } => {
            let value = MATE_SCORE - i32::from(plies);
            if winning { value } else { -value }
        }
    }
}

pub(super) fn move_text(movement: Move) -> String {
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
    visit_at_depths(fen, depths, &mut compare_case)
}

fn compare_case(position: &Position, depth: u8) -> TestResult {
    assert_same_as_plain_minimax(position, depth).map(|_| ())
}

fn visit_at_depths(
    fen: &str,
    depths: impl IntoIterator<Item = u8>,
    visit: &mut impl FnMut(&Position, u8) -> TestResult,
) -> TestResult {
    let position = Position::from_fen(fen)?;
    for depth in depths {
        visit(&position, depth)?;
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

/// The corpus exercised on every commit by the plain minimax comparison.
/// The deeper release-only cases below are run separately.
pub(super) fn visit_comparison_corpus(
    mut visit: impl FnMut(&Position, u8) -> TestResult,
) -> TestResult {
    visit_perft_positions(&mut visit)?;
    visit_seeded_walks(&mut visit)?;
    visit_examples(&mut visit)?;
    visit_open_scores(&mut visit)?;
    visit_published_mate_in_two(&mut visit)?;
    visit_published_mates_on_the_horizon(&mut visit)
}

fn visit_perft_positions(visit: &mut impl FnMut(&Position, u8) -> TestResult) -> TestResult {
    for record in records()? {
        visit_at_depths(record.fen, 1..=3, visit)?;
    }
    Ok(())
}

fn visit_seeded_walks(visit: &mut impl FnMut(&Position, u8) -> TestResult) -> TestResult {
    for seed in 1..=3 {
        for position in seeded_walk(seed)? {
            for depth in 1..=2 {
                visit(&position, depth)?;
            }
        }
    }
    Ok(())
}

fn visit_examples(visit: &mut impl FnMut(&Position, u8) -> TestResult) -> TestResult {
    for fen in EXAMPLES {
        visit_at_depths(fen, 1..=3, visit)?;
    }
    Ok(())
}

/// The positions where a transcript leaves the score open. The transcript
/// runner reads the same file and requires a position there for every
/// `{score}` at depths 1 to 4.
const OPEN_SCORES: &str = include_str!("../../../../test-data/open-scores.txt");

fn visit_open_scores(visit: &mut impl FnMut(&Position, u8) -> TestResult) -> TestResult {
    let mut positions = 0;
    for fen in OPEN_SCORES.lines() {
        if fen.is_empty() || fen.starts_with('#') {
            continue;
        }
        visit_at_depths(fen, 1..=4, visit)?;
        positions += 1;
    }
    assert!(positions > 0, "open-scores.txt lists no position");
    Ok(())
}

#[test]
fn perft_positions_at_depths_one_to_three() -> TestResult {
    visit_perft_positions(&mut compare_case)
}

#[test]
fn seeded_walks_at_depths_one_and_two() -> TestResult {
    visit_seeded_walks(&mut compare_case)
}

#[test]
fn examples_at_depths_one_to_three() -> TestResult {
    visit_examples(&mut compare_case)
}

#[test]
fn positions_whose_score_a_transcript_leaves_open() -> TestResult {
    visit_open_scores(&mut compare_case)
}

#[test]
fn prefix_check_rejects_a_variation_through_a_worse_reply() -> TestResult {
    // Perft position 2 at depth 3. The values come from the review of PR #76,
    // which reproduced them with the reference minimax on shakmaty before this
    // change removed it: cp 300 at the root, -300 for Black after e2a6, 600
    // for White after h8g8 and -300 for Black after a6c8. So e2a6 h8g8 a6c8
    // ends at cp 300 and passes the D5 replay, but h8g8 is a worse reply than
    // Black's best.
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let root = Position::from_fen(fen)?;
    let result = minimax(&root, 3);
    assert_eq!(result.score, Score::Centipawns(300));
    check_each_prefix(&root, 3, result.score, &result.principal_variation)?;

    let mut variation = Vec::new();
    let mut position = root;
    for text in ["e2a6", "h8g8", "a6c8"] {
        let movement = position
            .legal_moves()
            .into_iter()
            .find(|&movement| move_text(movement) == text)
            .ok_or("missing legal move")?;
        variation.push(movement);
        position = movement.play();
    }
    let chess = fen
        .parse::<Fen>()?
        .into_position::<Chess>(CastlingMode::Standard)?;
    principal_variation::verify(&chess, 3, 300, variation.iter().copied().map(move_text))?;
    let error = check_each_prefix(&root, 3, result.score, &variation)
        .err()
        .ok_or("the prefix check accepted e2a6 h8g8 a6c8")?;
    assert!(error.contains("after h8g8, 2 plies"), "{error}");
    assert!(error.contains("Centipawns(600)"), "{error}");
    Ok(())
}

fn visit_published_mate_in_two(visit: &mut impl FnMut(&Position, u8) -> TestResult) -> TestResult {
    let records = mate_in_two::records()?;
    let record = records
        .iter()
        .find(|record| record.id == "000Zo")
        .ok_or("missing puzzle 000Zo")?;
    visit_at_depths(record.fen, [4], visit)
}

fn visit_published_mates_on_the_horizon(
    visit: &mut impl FnMut(&Position, u8) -> TestResult,
) -> TestResult {
    // The checkmate is three plies away, so at depth 3 it stands on the
    // horizon and is scored by material alone.
    for record in mate_in_two::records()?.into_iter().take(10) {
        visit_at_depths(record.fen, [3], visit)?;
    }
    Ok(())
}

#[test]
fn published_mate_in_two_at_depth_four() -> TestResult {
    visit_published_mate_in_two(&mut compare_case)
}

#[test]
fn published_mates_in_two_on_the_horizon() -> TestResult {
    visit_published_mates_on_the_horizon(&mut compare_case)
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
