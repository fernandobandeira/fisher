#[path = "support/reference_minimax.rs"]
mod reference;

use std::error::Error;

use reference::minimax;
use shakmaty::{CastlingMode, Chess, Position as _, fen::Fen};

const PERFT: &str = include_str!("../../../expectations/perft.epd");

fn position(fen: &str) -> Result<Chess, Box<dyn Error>> {
    Ok(fen.parse::<Fen>()?.into_position(CastlingMode::Standard)?)
}

fn check_perft(depth: u8) -> Result<(), Box<dyn Error>> {
    let positions = PERFT
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'));

    for record in positions {
        let (fen, counts) = record.split_once(';').ok_or("missing perft counts")?;
        let fen = fen.trim();
        let mut expected = 0;
        for ply in 1..=depth {
            let label = format!("D{ply} ");
            let count = counts
                .split(';')
                .find_map(|entry| entry.trim().strip_prefix(&label))
                .ok_or_else(|| format!("missing perft count for {fen} at ply {ply}"))?;
            expected += count.parse::<u64>()?;
        }

        let (_, _, nodes) = minimax(&position(fen)?, depth);
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

#[test]
fn mate_in_one_scores_as_a_win() -> Result<(), Box<dyn Error>> {
    let position = position("k7/8/1QK5/8/8/8/8/8 w - - 0 1")?;
    // Qb7 gives check and covers a7 and b8. The white king protects b7.
    assert_eq!(minimax(&position, 2).1, 99_999);
    Ok(())
}

#[test]
fn stalemate_one_ply_away_is_avoided_before_the_horizon() -> Result<(), Box<dyn Error>> {
    let position = position("k7/8/1p6/1Q6/8/8/8/7K w - - 0 1")?;
    // Qxb6 leaves a8 unchecked, with a7, b7 and b8 covered. At depth 2,
    // stalemate scores zero, so White prefers retaining its 800-centipawn lead.
    // At depth 1, Qxb6 is evaluated as material and scores 900.
    assert_eq!(minimax(&position, 2).1, 800);
    assert_eq!(minimax(&position, 1).1, 900);
    Ok(())
}

#[test]
fn free_capture_scores_the_material_left() -> Result<(), Box<dyn Error>> {
    let position = position("7k/q7/8/8/8/8/8/R6K w - - 0 1")?;
    // Rxa7 wins the queen, leaving White a rook ahead.
    assert_eq!(minimax(&position, 1).1, 500);
    Ok(())
}

#[test]
fn checkmated_position_has_no_move_or_nodes() -> Result<(), Box<dyn Error>> {
    let position = position("k7/1Q6/2K5/8/8/8/8/8 b - - 0 1")?;
    assert_eq!(minimax(&position, 1), (None, -100_000, 0));
    Ok(())
}

#[test]
fn equal_scores_keep_the_first_legal_move() {
    let position = Chess::default();
    // No legal move from the starting position changes the material.
    let first = position.legal_moves().into_iter().next();
    assert_eq!(minimax(&position, 1).0, first);
}

#[test]
fn free_knight_capture_keeps_the_remaining_material_after_a_reply() -> Result<(), Box<dyn Error>> {
    let position = position("7k/n7/6r1/8/8/8/8/RNBQ3K w - - 0 1")?;
    // Rxa7 wins the knight. Black has legal replies but cannot capture any
    // white piece. The rooks cancel, leaving 300 + 300 + 900 centipawns.
    assert_eq!(minimax(&position, 2).1, 1_500);
    Ok(())
}
