#![cfg(test)]

use std::{collections::BTreeMap, error::Error};

use rules::{Color, PieceKind, Position};
use shakmaty::{CastlingMode, Chess, Position as _, Role, fen::Fen};

use super::{
    comparison::{move_text, visit_comparison_corpus},
    perft::records,
};
use crate::move_order::{MovePriority, capture_priority, piece_taken};

#[derive(Default)]
struct CaptureCoverage {
    white_en_passant: bool,
    black_en_passant: bool,
    capturing_promotion: bool,
}

#[test]
fn captures_and_the_pieces_they_take_agree_with_shakmaty() -> Result<(), Box<dyn Error>> {
    let mut coverage = CaptureCoverage::default();
    visit_comparison_corpus(|position, depth| {
        if depth == 1 {
            check_captures(position, &mut coverage)?;
            for movement in position.legal_moves() {
                check_captures(&movement.play(), &mut coverage)?;
            }
        }
        Ok(())
    })?;
    // The perft walks reach en passant for White at ply three.
    for record in records()? {
        let position = Position::from_fen(record.fen)?;
        for reply in position
            .legal_moves()
            .into_iter()
            .flat_map(|movement| movement.play().legal_moves())
        {
            check_captures(&reply.play(), &mut coverage)?;
        }
    }
    assert!(coverage.white_en_passant, "en passant for White");
    assert!(coverage.black_en_passant, "en passant for Black");
    assert!(
        coverage.capturing_promotion,
        "the corpus must cover a promotion that captures"
    );
    Ok(())
}

fn check_captures(
    position: &Position,
    coverage: &mut CaptureCoverage,
) -> Result<(), Box<dyn Error>> {
    let fen = position.to_fen();
    let chess = fen
        .parse::<Fen>()?
        .into_position::<Chess>(CastlingMode::Standard)?;
    let expected: BTreeMap<_, _> = chess
        .legal_moves()
        .into_iter()
        .filter_map(|movement| {
            if movement.is_en_passant() {
                match position.side_to_move() {
                    Color::White => coverage.white_en_passant = true,
                    Color::Black => coverage.black_en_passant = true,
                }
            }
            coverage.capturing_promotion |= movement.is_promotion() && movement.is_capture();
            (movement.is_capture() || movement.promotion() == Some(Role::Queen)).then(|| {
                (
                    movement.to_uci(CastlingMode::Standard).to_string(),
                    movement.capture().map(piece_kind),
                )
            })
        })
        .collect();
    let actual: BTreeMap<_, _> = position
        .legal_moves()
        .into_iter()
        .filter(|movement| capture_priority(position, movement) != MovePriority::Quiet)
        .map(|movement| (move_text(movement), piece_taken(position, &movement)))
        .collect();
    assert_eq!(actual, expected, "{fen}");
    Ok(())
}

fn piece_kind(role: Role) -> PieceKind {
    match role {
        Role::Pawn => PieceKind::Pawn,
        Role::Knight => PieceKind::Knight,
        Role::Bishop => PieceKind::Bishop,
        Role::Rook => PieceKind::Rook,
        Role::Queen => PieceKind::Queen,
        Role::King => PieceKind::King,
    }
}
