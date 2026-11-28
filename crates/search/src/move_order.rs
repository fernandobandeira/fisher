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
            (Reverse(priority), index, movement)
        })
        .collect();
    order.sort_unstable_by_key(|&(priority, index, _)| (priority, index));
    order
        .into_iter()
        .map(move |(Reverse(priority), index, movement)| {
            let replies = (priority == MovePriority::Remembered)
                .then(|| previous.get(1..).unwrap_or_default());
            (index, *movement, replies)
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
    position
        .piece_at(movement.from())
        .is_some_and(|piece| {
            piece.kind() == PieceKind::Pawn && is_en_passant(movement, piece.color())
        })
        .then_some(PieceKind::Pawn)
}

/// A legal pawn move to an empty square on the en passant destination rank
/// either advances from the same file or captures en passant from another.
fn is_en_passant(movement: &Move, color: Color) -> bool {
    let squares = match color {
        Color::White => [
            ("a5", "a6"),
            ("b5", "b6"),
            ("c5", "c6"),
            ("d5", "d6"),
            ("e5", "e6"),
            ("f5", "f6"),
            ("g5", "g6"),
            ("h5", "h6"),
        ],
        Color::Black => [
            ("a4", "a3"),
            ("b4", "b3"),
            ("c4", "c3"),
            ("d4", "d3"),
            ("e4", "e3"),
            ("f4", "f3"),
            ("g4", "g3"),
            ("h4", "h3"),
        ],
    };
    squares.into_iter().any(|(from, to)| {
        Square::parse(to) == Some(movement.to()) && Square::parse(from) != Some(movement.from())
    })
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
