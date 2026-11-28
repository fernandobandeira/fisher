use std::cmp::Reverse;

use rules::{Color, Move, PieceKind, Position, Promotion, Square};

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum MovePriority {
    Quiet,
    QueenPromotion,
    Capture { taken: u8, taking: Reverse<u8> },
    Remembered,
}

pub(super) fn ordered_moves<'a>(
    position: &Position,
    moves: &'a [Move],
    previous: &'a [Move],
) -> impl Iterator<Item = (usize, Move, Option<&'a [Move]>)> + 'a {
    let mut order: Vec<_> = moves
        .iter()
        .enumerate()
        .map(|(index, movement)| {
            let remembered = previous.first().is_some_and(|prior| {
                movement.from() == prior.from()
                    && movement.to() == prior.to()
                    && movement.promotion() == prior.promotion()
            });
            let priority = if remembered {
                MovePriority::Remembered
            } else {
                capture_priority(position, movement)
            };
            (Reverse(priority), index)
        })
        .collect();
    order.sort_unstable();
    order
        .into_iter()
        .filter_map(move |(Reverse(priority), index)| {
            let movement = *moves.get(index)?;
            let replies = (priority == MovePriority::Remembered)
                .then(|| previous.get(1..).unwrap_or_default());
            Some((index, movement, replies))
        })
}

pub(super) fn capture_priority(position: &Position, movement: &Move) -> MovePriority {
    if let Some(taken) = piece_taken(position, movement) {
        let taking = position
            .piece_at(movement.from())
            .map_or(0, |piece| piece_value(piece.kind()));
        return MovePriority::Capture {
            taken: piece_value(taken),
            taking: Reverse(taking),
        };
    }
    // Stockfish's capture stage gives a queen promotion on an empty square
    // zero captured-piece value. The other promotions on empty squares are quiet.
    if movement.promotion() == Some(Promotion::Queen) {
        MovePriority::QueenPromotion
    } else {
        MovePriority::Quiet
    }
}

pub(super) fn piece_taken(position: &Position, movement: &Move) -> Option<PieceKind> {
    if let Some(piece) = position.piece_at(movement.to()) {
        return Some(piece.kind());
    }
    let piece = position.piece_at(movement.from())?;
    (piece.kind() == PieceKind::Pawn && is_en_passant(movement, piece.color()))
        .then_some(PieceKind::Pawn)
}

/// A legal pawn move to an empty square on the en passant destination rank
/// either advances from the same file or captures en passant from another.
fn is_en_passant(movement: &Move, color: Color) -> bool {
    // Square::all walks a1 through h8, one rank at a time.
    let (from, to) = match color {
        Color::White => (4, 5),
        Color::Black => (3, 2),
    };
    Square::all()
        .skip(8 * from)
        .zip(Square::all().skip(8 * to))
        .take(8)
        .any(|(from, to)| movement.to() == to && movement.from() != from)
}

fn piece_value(kind: PieceKind) -> u8 {
    match kind {
        PieceKind::Pawn => 1,
        PieceKind::Knight | PieceKind::Bishop => 2,
        PieceKind::Rook => 3,
        PieceKind::Queen => 4,
        PieceKind::King => 5,
    }
}
