#![cfg(test)]

use std::error::Error;

use rules::Position;
use shakmaty::{CastlingMode, Chess, EnPassantMode, Position as _, fen::Fen, uci::UciMove};

use super::comparison::move_text;

// D68, reasoned by hand from the rules and material. In each position White's
// rook can take the undefended queen on a7. The knight on g6 in the second
// position remains on the board after Rxa7, changing the material at depth 1.
const FIRST: &str = "7k/q7/8/8/8/8/8/R6K w - - 0 1";
const SECOND: &str = "7k/q7/6n1/8/8/8/8/R6K w - - 0 1";
const SHARED_MOVE: &str = "a1a7";

#[test]
fn the_same_origin_destination_and_promotion_lead_to_different_positions()
-> Result<(), Box<dyn Error>> {
    let first = position_after_shared_move(FIRST)?;
    let second = position_after_shared_move(SECOND)?;
    assert_ne!(first, second);
    Ok(())
}

fn position_after_shared_move(fen: &str) -> Result<Position, Box<dyn Error>> {
    let root = Position::from_fen(fen)?;
    let movement = root
        .legal_moves()
        .into_iter()
        .find(|&movement| move_text(movement) == SHARED_MOVE)
        .ok_or_else(|| format!("{SHARED_MOVE} is not legal at {fen}"))?;
    let chess = fen
        .parse::<Fen>()?
        .into_position::<Chess>(CastlingMode::Standard)?;
    let reference_move = SHARED_MOVE.parse::<UciMove>()?.to_move(&chess)?;
    let reference = chess.play(reference_move)?;

    let next = movement.play();
    assert_eq!(
        next.to_fen(),
        Fen::from_position(&reference, EnPassantMode::Always).to_string(),
        "{fen} after {SHARED_MOVE}"
    );
    Ok(next)
}
