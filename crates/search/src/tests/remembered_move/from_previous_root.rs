#![cfg(test)]
//! D68, staged for fisher-707.4. Add `mod from_previous_root;` to
//! `remembered_move.rs` when the interruptible search takes `MoveOrderMemory`.
//! `MoveOrderMemory::default()` stands in for the empty constructor.

use std::{error::Error, num::NonZeroU8};

use rules::Position;

use super::{
    super::{
        comparison::{check_each_prefix, move_text},
        plain_minimax::plain_minimax,
    },
    FIRST, SECOND, SHARED_MOVE,
};
use crate::{MoveOrderMemory, minimax_variations_until_interrupted};

#[test]
fn the_remembered_move_is_played_from_the_position_in_hand() -> Result<(), Box<dyn Error>> {
    let first = Position::from_fen(FIRST)?;
    let second = Position::from_fen(SECOND)?;
    let depth = 1;
    let mut remembered = MoveOrderMemory::default();
    let previous = minimax_variations_until_interrupted(
        &first,
        depth,
        NonZeroU8::MIN,
        &mut remembered,
        || false,
    )
    .ok_or("first search did not finish")?;
    let previous_root_move = previous
        .variations
        .first()
        .and_then(|(_, moves)| moves.first());
    assert_eq!(
        previous_root_move.copied().map(move_text).as_deref(),
        Some(SHARED_MOVE),
        "the first search must remember the shared move"
    );

    let result = minimax_variations_until_interrupted(
        &second,
        depth,
        NonZeroU8::MIN,
        &mut remembered,
        || false,
    )
    .ok_or("second search did not finish")?;
    let expected = plain_minimax(&second, depth);
    let (score, moves) = result
        .variations
        .first()
        .ok_or("missing principal variation")?;
    assert_eq!(result.score, expected.score, "{SECOND}");
    assert_eq!(*score, expected.score, "first variation score at {SECOND}");
    assert!(!moves.is_empty(), "missing root move at {SECOND}");
    check_each_prefix(&second, depth, result.score, moves)?;
    Ok(())
}
