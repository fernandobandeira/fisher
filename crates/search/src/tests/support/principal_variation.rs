#![cfg(test)]
//! Replays a principal variation with shakmaty and checks the property of D5
//! in fisher-r6y. It uses no part of fisher.

use shakmaty::{Chess, EnPassantMode, Position as _, Role, fen::Fen, uci::UciMove};

/// The magnitude of a score for checkmate at the root. A score for checkmate
/// `plies` away is `MATE_SCORE - plies`, signed for the root's side to move.
/// It is greater than any material difference, even with promotions.
pub(super) const MATE_SCORE: i32 = 100_000;

/// Checks that every move of `variation` is legal in turn, and that the
/// variation supports `score`, from the side to move at `start`:
///
/// - a variation of `depth` plies ends where the material count equals the score;
/// - a shorter one ends in checkmate at the reported distance, or in stalemate
///   with a score of 0.
pub(super) fn verify(
    start: &Chess,
    depth: u8,
    score: i32,
    variation: impl IntoIterator<Item: AsRef<str>>,
) -> Result<(), String> {
    let fen = Fen::from_position(start, EnPassantMode::Always);
    let mut position = start.clone();
    let mut ply = 0;
    for movement in variation {
        let text = movement.as_ref();
        let legal = position
            .legal_moves()
            .into_iter()
            .find(|&legal| UciMove::from_standard(legal).to_string() == text)
            .ok_or_else(|| {
                format!("{fen}: illegal principal variation move {text} after {ply} plies")
            })?;
        position.play_unchecked(legal);
        ply += 1;
        if ply > depth {
            return Err(format!("{fen}: principal variation exceeds depth {depth}"));
        }
    }

    if is_mate(score) {
        let expected = MATE_SCORE - score.abs();
        if i32::from(ply) != expected {
            return Err(format!(
                "{fen}: principal variation expected {expected} plies, got {ply}"
            ));
        }
        if ply == depth || !position.is_checkmate() || (score < 0) != ply.is_multiple_of(2) {
            return Err(format!(
                "{fen}: principal variation does not end in the reported mate"
            ));
        }
    } else if ply < depth {
        if !(score == 0 && position.is_stalemate()) {
            return Err(format!(
                "{fen}: principal variation expected {depth} plies, got {ply}"
            ));
        }
    } else {
        let at_end = material(&position);
        let from_start = if ply.is_multiple_of(2) {
            at_end
        } else {
            -at_end
        };
        if from_start != score {
            return Err(format!(
                "{fen}: principal variation ends with material {from_start}, not the score {score}"
            ));
        }
    }
    Ok(())
}

fn is_mate(score: i32) -> bool {
    score.unsigned_abs() >= (MATE_SCORE - i32::from(u8::MAX)).unsigned_abs()
}

/// The material count for the side to move, in centipawns.
fn material(position: &Chess) -> i32 {
    [
        (Role::Pawn, 100),
        (Role::Knight, 300),
        (Role::Bishop, 300),
        (Role::Rook, 500),
        (Role::Queen, 900),
    ]
    .into_iter()
    .map(|(kind, value)| {
        let ours: i32 = position.our(kind).into_iter().map(|_| value).sum();
        let theirs: i32 = position.their(kind).into_iter().map(|_| value).sum();
        ours - theirs
    })
    .sum()
}
