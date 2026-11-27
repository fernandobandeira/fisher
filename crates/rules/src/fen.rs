use std::{fmt, num::NonZeroU64};

use crate::{Castle, CastlingRights, Color, Piece, PieceKind, Position, Square};

const CASTLING_RIGHTS: [(char, Color, Castle); 4] = [
    ('K', Color::White, Castle::Kingside),
    ('Q', Color::White, Castle::Queenside),
    ('k', Color::Black, Castle::Kingside),
    ('q', Color::Black, Castle::Queenside),
];

/// Explains why a FEN cannot describe a position for the engine.
#[derive(Debug, Eq, PartialEq)]
pub struct FenError(&'static str);

impl fmt::Display for FenError {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        output.write_str(self.0)
    }
}

impl std::error::Error for FenError {}

impl Position {
    /// Reads a full six-field FEN.
    ///
    /// # Errors
    ///
    /// Reports an invalid field, invalid piece placement, or a position in which the color
    /// that is not the side to move is in check.
    pub fn from_fen(fen: &str) -> Result<Self, FenError> {
        let fields: Vec<_> = fen.split_whitespace().collect();
        let [pieces, color, castling, en_passant, halfmove, fullmove] = fields.as_slice() else {
            return Err(FenError("FEN must have six fields"));
        };
        let side_to_move = match *color {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return Err(FenError("side to move must be w or b")),
        };
        let mut position = Self {
            pieces: [None; 64],
            side_to_move,
            castling_rights: parse_castling_rights(castling)?,
            en_passant: parse_en_passant(en_passant, side_to_move)?,
            halfmove_clock: decimal(halfmove).ok_or(FenError(
                "halfmove clock must be an unsigned decimal integer",
            ))?,
            fullmove_number: decimal(fullmove).and_then(NonZeroU64::new).ok_or(FenError(
                "fullmove number must be a positive decimal integer",
            ))?,
        };
        position.read_pieces(pieces)?;
        position.check_pieces()?;
        Ok(position)
    }

    /// Writes the canonical FEN, including an en passant square even without a legal capture.
    #[must_use]
    pub fn to_fen(&self) -> String {
        format!(
            "{} {} {} {} {} {}",
            self.write_pieces(),
            write_side_to_move(self.side_to_move),
            write_castling_rights(self.castling_rights),
            write_en_passant(self.en_passant),
            self.halfmove_clock,
            self.fullmove_number,
        )
    }

    fn read_pieces(&mut self, text: &str) -> Result<(), FenError> {
        let ranks = text.split('/');
        if ranks.clone().count() != 8 {
            return Err(FenError("piece placement must have eight ranks"));
        }
        for (rank_text, rank) in ranks.zip((0..8).rev()) {
            self.read_rank(rank_text, rank)?;
        }
        Ok(())
    }

    fn read_rank(&mut self, text: &str, rank: u8) -> Result<(), FenError> {
        const WRONG_WIDTH: FenError = FenError("each rank must have eight files");
        let mut file = 0;
        for byte in text.bytes() {
            if file >= 8 {
                return Err(WRONG_WIDTH);
            }
            match byte {
                b'1'..=b'8' => file += byte - b'0',
                _ => {
                    let piece = read_piece(byte).ok_or(FenError("unknown piece letter in FEN"))?;
                    self.put(Square::new(file, rank), piece);
                    file += 1;
                }
            }
        }
        if file != 8 {
            return Err(WRONG_WIDTH);
        }
        Ok(())
    }

    fn write_pieces(&self) -> String {
        let mut text = String::new();
        for rank in (0..8).rev() {
            if !text.is_empty() {
                text.push('/');
            }
            self.write_rank(rank, &mut text);
        }
        text
    }

    // A rank with only a white pawn on the e-file reads as 4P3.
    fn write_rank(&self, rank: u8, text: &mut String) {
        let mut empty_squares = 0;
        for file in 0..8 {
            let Some(piece) = self.piece_at(Square::new(file, rank)) else {
                empty_squares += 1;
                continue;
            };
            write_empty_squares(empty_squares, text);
            text.push(piece_letter(piece));
            empty_squares = 0;
        }
        write_empty_squares(empty_squares, text);
    }

    fn check_pieces(&self) -> Result<(), FenError> {
        let white_king = self.fen_king(Color::White)?;
        let black_king = self.fen_king(Color::Black)?;
        if white_king.is_adjacent_to(black_king) {
            return Err(FenError("a position cannot have adjacent kings"));
        }
        let king_not_to_move = match self.side_to_move {
            Color::White => black_king,
            Color::Black => white_king,
        };
        if self.is_attacked(king_not_to_move, self.side_to_move) {
            return Err(FenError(
                "the color that is not the side to move cannot be in check",
            ));
        }
        if self.has_back_rank_pawn() {
            return Err(FenError(
                "a position cannot have a pawn on rank 1 or rank 8",
            ));
        }
        self.check_pawn_count()?;
        for (_, color, castle) in CASTLING_RIGHTS {
            if self.castling_rights.has(color, castle)
                && !self.has_castling_pieces_at_home(color, castle)
            {
                return Err(FenError(
                    "castling rights require the king and rook on their home squares",
                ));
            }
        }
        self.check_en_passant()
    }

    fn check_pawn_count(&self) -> Result<(), FenError> {
        let mut pawns = [0; 2];
        for piece in self.pieces.iter().flatten() {
            if piece.kind == PieceKind::Pawn {
                let count = match piece.color {
                    Color::White => &mut pawns[0],
                    Color::Black => &mut pawns[1],
                };
                *count += 1;
            }
        }
        if pawns.into_iter().any(|count| count > 8) {
            return Err(FenError("a color cannot have more than eight pawns"));
        }
        Ok(())
    }

    fn check_en_passant(&self) -> Result<(), FenError> {
        let Some(target) = self.en_passant else {
            return Ok(());
        };
        let (color, origin_rank, destination_rank) = match self.side_to_move {
            Color::White => (Color::Black, 6, 4),
            Color::Black => (Color::White, 1, 3),
        };
        let origin = Square::new(target.file.0, origin_rank);
        let destination = Square::new(target.file.0, destination_rank);
        if self.piece_at(target).is_some()
            || self.piece_at(origin).is_some()
            || !self.has_piece_at(destination, color, PieceKind::Pawn)
        {
            return Err(FenError(
                "en passant requires a pawn on its destination and empty origin and target squares",
            ));
        }
        Ok(())
    }

    fn has_back_rank_pawn(&self) -> bool {
        Square::all().any(|square| {
            matches!(square.rank.0, 0 | 7)
                && self
                    .piece_at(square)
                    .is_some_and(|piece| piece.kind == PieceKind::Pawn)
        })
    }

    fn has_castling_pieces_at_home(&self, color: Color, castle: Castle) -> bool {
        self.has_piece_at(color.king_home(), color, PieceKind::King)
            && self.has_piece_at(castle.rook_home(color), color, PieceKind::Rook)
    }

    fn fen_king(&self, color: Color) -> Result<Square, FenError> {
        let mut kings =
            Square::all().filter(|&square| self.has_piece_at(square, color, PieceKind::King));
        if let (Some(king), None) = (kings.next(), kings.next()) {
            Ok(king)
        } else {
            Err(FenError(match color {
                Color::White => "FEN must have exactly one king for White",
                Color::Black => "FEN must have exactly one king for Black",
            }))
        }
    }
}

fn write_empty_squares(count: u8, text: &mut String) {
    if count > 0 {
        text.push(char::from(b'0' + count));
    }
}

const fn write_side_to_move(color: Color) -> char {
    match color {
        Color::White => 'w',
        Color::Black => 'b',
    }
}

fn write_castling_rights(rights: CastlingRights) -> String {
    let mut text = String::new();
    for (letter, color, castle) in CASTLING_RIGHTS {
        if rights.has(color, castle) {
            text.push(letter);
        }
    }
    if text.is_empty() {
        text.push('-');
    }
    text
}

fn write_en_passant(target: Option<Square>) -> String {
    match target {
        Some(square) => square.to_string(),
        None => "-".to_owned(),
    }
}

fn parse_castling_rights(text: &str) -> Result<CastlingRights, FenError> {
    let mut rights = CastlingRights::NONE;
    if text == "-" {
        return Ok(rights);
    }
    for letter in text.chars() {
        let (_, color, castle) = CASTLING_RIGHTS
            .into_iter()
            .find(|&(candidate, _, _)| letter == candidate)
            .ok_or(FenError("castling rights must use K, Q, k, q, or -"))?;
        if rights.has(color, castle) {
            return Err(FenError("castling rights must not contain duplicates"));
        }
        rights.insert(color, castle);
    }
    Ok(rights)
}

fn parse_en_passant(text: &str, side_to_move: Color) -> Result<Option<Square>, FenError> {
    if text == "-" {
        return Ok(None);
    }
    let square = Square::parse(text).ok_or(FenError("invalid en passant square"))?;
    let target_rank = match side_to_move {
        Color::White => 5,
        Color::Black => 2,
    };
    if square.rank.0 != target_rank {
        return Err(FenError(
            "en passant square has the wrong rank for the side to move",
        ));
    }
    Ok(Some(square))
}

fn decimal(text: &str) -> Option<u64> {
    // str::parse alone accepts a leading +.
    if !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

fn read_piece(byte: u8) -> Option<Piece> {
    let color = if byte.is_ascii_uppercase() {
        Color::White
    } else {
        Color::Black
    };
    let kind = [
        PieceKind::Pawn,
        PieceKind::Knight,
        PieceKind::Bishop,
        PieceKind::Rook,
        PieceKind::Queen,
        PieceKind::King,
    ]
    .into_iter()
    .find(|&kind| piece_kind_letter(kind) == char::from(byte.to_ascii_lowercase()))?;
    Some(Piece { color, kind })
}

fn piece_letter(piece: Piece) -> char {
    let letter = piece_kind_letter(piece.kind);
    match piece.color {
        Color::White => letter.to_ascii_uppercase(),
        Color::Black => letter,
    }
}

const fn piece_kind_letter(kind: PieceKind) -> char {
    match kind {
        PieceKind::Pawn => 'p',
        PieceKind::Knight => 'n',
        PieceKind::Bishop => 'b',
        PieceKind::Rook => 'r',
        PieceKind::Queen => 'q',
        PieceKind::King => 'k',
    }
}
