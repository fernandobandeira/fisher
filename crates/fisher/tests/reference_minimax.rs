#[path = "support/mate_in_two.rs"]
mod mate_in_two;
#[path = "support/reference_minimax.rs"]
mod reference;
mod support;

use std::error::Error;

use reference::minimax;
use shakmaty::{Chess, Position as _, uci::UciMove};
use support::{check_perft, reference_position as position};

fn reference_nodes(fen: &str, depth: u8) -> Result<u64, Box<dyn Error>> {
    Ok(minimax(&position(fen)?, depth).nodes)
}

#[test]
fn node_count_matches_published_perft_at_two_plies() -> Result<(), Box<dyn Error>> {
    check_perft(2, reference_nodes)
}

#[test]
#[ignore = "run in release mode for the published four-ply counts"]
fn node_count_matches_published_perft_at_four_plies() -> Result<(), Box<dyn Error>> {
    check_perft(4, reference_nodes)
}

#[test]
fn mate_in_one_scores_as_a_win() -> Result<(), Box<dyn Error>> {
    let position = position("k7/8/1QK5/8/8/8/8/8 w - - 0 1")?;
    // Qb7 gives check and covers a7 and b8. The white king protects b7.
    assert_eq!(minimax(&position, 2).score, 99_999);
    Ok(())
}

#[test]
fn stalemate_one_ply_away_is_avoided_before_the_horizon() -> Result<(), Box<dyn Error>> {
    let position = position("k7/8/1p6/1Q6/8/8/8/7K w - - 0 1")?;
    // Qxb6 leaves a8 unchecked, with a7, b7 and b8 covered. At depth 2,
    // stalemate scores zero, so White prefers retaining its 800-centipawn lead.
    // At depth 1, Qxb6 is evaluated as material and scores 900.
    assert_eq!(minimax(&position, 2).score, 800);
    assert_eq!(minimax(&position, 1).score, 900);
    Ok(())
}

#[test]
fn free_capture_scores_the_material_left() -> Result<(), Box<dyn Error>> {
    let position = position("7k/q7/8/8/8/8/8/R6K w - - 0 1")?;
    // Rxa7 wins the queen, leaving White a rook ahead.
    assert_eq!(minimax(&position, 1).score, 500);
    Ok(())
}

#[test]
fn checkmated_position_has_no_move_or_nodes() -> Result<(), Box<dyn Error>> {
    let position = position("k7/1Q6/2K5/8/8/8/8/8 b - - 0 1")?;
    let result = minimax(&position, 1);
    assert_eq!(result.chosen_move, None);
    assert_eq!(result.score, -100_000);
    assert_eq!(result.nodes, 0);
    Ok(())
}

#[test]
fn equal_scores_keep_the_first_legal_move() {
    let position = Chess::default();
    // No legal move from the starting position changes the material.
    let first = position.legal_moves().into_iter().next();
    assert_eq!(minimax(&position, 1).chosen_move, first);
}

#[test]
fn free_knight_capture_keeps_the_remaining_material_after_a_reply() -> Result<(), Box<dyn Error>> {
    let position = position("7k/n7/6r1/8/8/8/8/RNBQ3K w - - 0 1")?;
    // Rxa7 wins the knight. Black has legal replies but cannot capture any
    // white piece. The rooks cancel, leaving 300 + 300 + 900 centipawns.
    assert_eq!(minimax(&position, 2).score, 1_500);
    Ok(())
}

#[test]
fn quiet_move_beats_a_capture_that_allows_a_recapture() -> Result<(), Box<dyn Error>> {
    let position = position("7k/8/2p5/3p4/8/8/8/3Q3K w - - 0 1")?;
    // White starts 900 - 200 = 700 centipawns ahead. Qxd5 is the only capture.
    // At depth 1 it scores the material left, 900 - 100 = 800. At depth 2,
    // cxd5 answers it and leaves Black a pawn ahead, -100. A quiet move such as
    // Qd2 keeps the 700 lead: no black piece attacks the queen there, and no
    // white move checkmates or stalemates Black.
    assert_eq!(minimax(&position, 1).score, 800);
    assert_eq!(minimax(&position, 2).score, 700);
    Ok(())
}

#[test]
fn published_mate_in_two_scores_as_a_win_with_the_published_move() -> Result<(), Box<dyn Error>> {
    // Lichess puzzle 000Zo is mate in two, three plies from this position, so
    // it scores 100_000 - 3. The README says exactly one legal move forces
    // that checkmate, so the first best move is the published move, e8e1.
    let expected_score = 99_997;
    let records = mate_in_two::records()?;
    let record = records
        .iter()
        .find(|record| record.id == "000Zo")
        .ok_or("missing puzzle 000Zo")?;

    let result = minimax(&position(record.fen)?, 4);

    assert_eq!(result.score, expected_score);
    let best_move = result
        .chosen_move
        .map(|movement| UciMove::from_standard(movement).to_string());
    assert_eq!(best_move.as_deref(), Some(record.published_move));
    Ok(())
}
