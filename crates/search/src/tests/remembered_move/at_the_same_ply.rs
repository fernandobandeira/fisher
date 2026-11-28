#![cfg(test)]

use std::{error::Error, num::NonZeroU8};

use rules::{Move, Position};
use shakmaty::{CastlingMode, Chess, EnPassantMode, Position as _, fen::Fen, uci::UciMove};

use crate::tests::{
    comparison::{check_each_prefix, move_text, score_value},
    plain_minimax::plain_minimax,
    principal_variation,
};
use crate::{MoveOrderMemory, minimax_variations_until_interrupted};

// D68, reasoned by hand from the rules and material. At depth 3, Bxb8 Kxb8
// trades bishop for knight, leaving White a rook against a pawn (+400).
// The search reaches Rb4 before Ra4+. After Rb4, the quiet Na6+ saves the
// knight and can cut off at +400 for White. After Ra4+, Na6+ blocks the check
// but White can play Rxa6. Black's other reply, Kb7, allows Bxb8. Either
// gives White +700. Replaying the move stored after Rb4 puts the rook back
// on b4, where it cannot take the knight, and wrongly cuts off Ra4+.
// White's king on c5 blocks Rc8, and the pawn on d7 prevents Nd7.
const ROOT: &str = "kn6/3p4/8/2K5/2R5/6B1/8/8 w - - 0 1";
const FIRST_ROOT_MOVE: &str = "c4b4";
const SECOND_ROOT_MOVE: &str = "c4a4";
const SHARED_REPLY: &str = "b8a6";

type TestResult = Result<(), Box<dyn Error>>;

#[test]
fn the_same_quiet_reply_leads_to_different_positions_and_scores() -> TestResult {
    let root = Position::from_fen(ROOT)?;
    let first = quiet_reply_after(&root, FIRST_ROOT_MOVE)?;
    let second = quiet_reply_after(&root, SECOND_ROOT_MOVE)?;
    assert_eq!(
        (first.from(), first.to(), first.promotion()),
        (second.from(), second.to(), second.promotion())
    );
    assert_ne!(first.play(), second.play());
    assert_ne!(
        plain_minimax(&first.play(), 1).score,
        plain_minimax(&second.play(), 1).score
    );
    Ok(())
}

fn quiet_reply_after(root: &Position, root_move: &str) -> Result<Move, Box<dyn Error>> {
    let position = legal_move(root, root_move)?.play();
    let reply = legal_move(&position, SHARED_REPLY)?;
    let chess = position
        .to_fen()
        .parse::<Fen>()?
        .into_position::<Chess>(CastlingMode::Standard)?;
    let reference = SHARED_REPLY.parse::<UciMove>()?.to_move(&chess)?;
    assert!(!reference.is_capture());
    assert_eq!(reference.promotion(), None);
    assert_eq!(
        reply.play().to_fen(),
        Fen::from_position(&chess.play(reference)?, EnPassantMode::Always).to_string()
    );
    Ok(reply)
}

fn legal_move(position: &Position, text: &str) -> Result<Move, String> {
    position
        .legal_moves()
        .into_iter()
        .find(|&movement| move_text(movement) == text)
        .ok_or_else(|| format!("{text} is not legal at {}", position.to_fen()))
}

#[test]
fn a_remembered_reply_keeps_the_score_with_fresh_memory_at_each_depth() -> TestResult {
    let root = Position::from_fen(ROOT)?;
    for depth in 1..=3 {
        check_depth(&root, depth, &mut MoveOrderMemory::default())?;
    }
    Ok(())
}

#[test]
fn a_remembered_reply_keeps_the_score_with_memory_carried_across_depths() -> TestResult {
    let root = Position::from_fen(ROOT)?;
    let mut memory = MoveOrderMemory::default();
    for depth in 1..=3 {
        check_depth(&root, depth, &mut memory)?;
    }
    Ok(())
}

fn check_depth(root: &Position, depth: u8, memory: &mut MoveOrderMemory) -> TestResult {
    let result =
        minimax_variations_until_interrupted(root, depth, NonZeroU8::MIN, memory, || false)
            .ok_or("search did not finish")?;
    let expected = plain_minimax(root, depth);
    assert_eq!(result.score, expected.score, "score at depth {depth}");
    let (score, moves) = result
        .variations
        .first()
        .ok_or("missing principal variation")?;
    assert_eq!(*score, expected.score, "variation score at depth {depth}");
    let chess = ROOT
        .parse::<Fen>()?
        .into_position::<Chess>(CastlingMode::Standard)?;
    principal_variation::verify(
        &chess,
        depth,
        score_value(*score),
        moves.iter().copied().map(move_text),
    )?;
    check_each_prefix(root, depth, *score, moves)?;
    Ok(())
}
