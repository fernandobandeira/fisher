use shakmaty::{Chess, EnPassantMode, Position as _, fen::Fen, uci::UciMove};

use super::reference;

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

        let reply = reference::minimax(&position, depth - ply).score;
        // The reference call counts mate distance from its own starting position.
        let reply = if is_mate(reply) {
            reply - reply.signum() * i32::from(ply)
        } else {
            reply
        };
        let from_start = if ply.is_multiple_of(2) { reply } else { -reply };
        if from_start != score {
            return Err(format!(
                "{fen}: {text} changes the best score from {score} to {from_start} at ply {ply}"
            ));
        }
    }

    if is_mate(score) {
        let expected = reference::MATE_SCORE - score.abs();
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
    } else if ply < depth && !(score == 0 && position.is_stalemate()) {
        return Err(format!(
            "{fen}: principal variation expected {depth} plies, got {ply}"
        ));
    }
    Ok(())
}

fn is_mate(score: i32) -> bool {
    score.unsigned_abs() >= (reference::MATE_SCORE - i32::from(u8::MAX)).unsigned_abs()
}
