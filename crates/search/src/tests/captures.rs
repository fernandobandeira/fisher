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

#[test]
fn capture_priorities_favor_the_piece_taken_then_the_piece_taking() -> Result<(), Box<dyn Error>> {
    let position = Position::from_fen("7k/8/3q4/8/3Rp3/8/5N2/K7 w - - 0 1")?;
    let rook_takes_queen = priority(&position, "d4d6")?;
    let rook_takes_pawn = priority(&position, "d4e4")?;
    let knight_takes_pawn = priority(&position, "f2e4")?;

    assert!(rook_takes_queen > rook_takes_pawn);
    assert!(knight_takes_pawn > rook_takes_pawn);
    Ok(())
}

#[test]
fn queen_promotion_and_en_passant_have_capture_priorities() -> Result<(), Box<dyn Error>> {
    let position = Position::from_fen("7k/P7/1p6/2PpP3/8/8/8/K7 w - d6 0 1")?;
    let pawn_takes_pawn = priority(&position, "c5b6")?;
    let en_passant = priority(&position, "e5d6")?;
    let queen_promotion = priority(&position, "a7a8q")?;
    let quiet_move = priority(&position, "a1b1")?;

    assert!(en_passant == pawn_takes_pawn);
    assert!(pawn_takes_pawn > queen_promotion);
    assert!(queen_promotion > quiet_move);
    Ok(())
}

fn priority(position: &Position, text: &str) -> Result<MovePriority, String> {
    position
        .legal_moves()
        .iter()
        .find(|&&movement| move_text(movement) == text)
        .map(|movement| capture_priority(position, movement))
        .ok_or_else(|| format!("{text} is not legal in {}", position.to_fen()))
}
