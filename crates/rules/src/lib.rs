use std::{fmt, num::NonZeroU64};

mod fen;
mod legal_moves;

pub use fen::FenError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Color {
    White,
    Black,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PieceKind {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Piece {
    color: Color,
    kind: PieceKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct File(u8);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Rank(u8);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Square {
    file: File,
    rank: Rank,
}

impl Square {
    /// Reads a square name such as `e4`.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        // Store file and rank as offsets from a1. For e4, 'e' - 'a' gives file 4
        // and '4' - '1' gives rank 3, so the square holds (4, 3).
        match *text.as_bytes() {
            [file @ b'a'..=b'h', rank @ b'1'..=b'8'] => Some(Self::new(file - b'a', rank - b'1')),
            _ => None,
        }
    }

    const fn new(file: u8, rank: u8) -> Self {
        Self {
            file: File(file),
            rank: Rank(rank),
        }
    }

    fn index(self) -> usize {
        usize::from(self.rank.0) * 8 + usize::from(self.file.0)
    }

    fn is_adjacent_to(self, other: Self) -> bool {
        self != other
            && self.file.0.abs_diff(other.file.0) <= 1
            && self.rank.0.abs_diff(other.rank.0) <= 1
    }

    fn offset(self, file: i8, rank: i8) -> Option<Self> {
        let file = self
            .file
            .0
            .checked_add_signed(file)
            .filter(|&value| value < 8)?;
        let rank = self
            .rank
            .0
            .checked_add_signed(rank)
            .filter(|&value| value < 8)?;
        Some(Self::new(file, rank))
    }

    fn all() -> impl Iterator<Item = Self> {
        (0..8).flat_map(|rank| (0..8).map(move |file| Self::new(file, rank)))
    }
}

impl fmt::Display for Square {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            output,
            "{}{}",
            char::from(b'a' + self.file.0),
            char::from(b'1' + self.rank.0)
        )
    }
}

/// The four piece kinds a pawn can promote to.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Promotion {
    Knight,
    Bishop,
    Rook,
    Queen,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Move {
    from: Square,
    to: Square,
    promotion: Option<Promotion>,
    position: Position,
}

impl Move {
    #[must_use]
    pub fn from(self) -> Square {
        self.from
    }

    #[must_use]
    pub fn to(self) -> Square {
        self.to
    }

    #[must_use]
    pub fn promotion(self) -> Option<Promotion> {
        self.promotion
    }

    /// Returns the position reached by playing this legal move.
    #[must_use]
    pub fn play(self) -> Position {
        self.position
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Position {
    pieces: [Option<Piece>; 64],
    side_to_move: Color,
    castling_rights: CastlingRights,
    en_passant: Option<Square>,
    halfmove_clock: u64,
    fullmove_number: NonZeroU64,
}

impl Position {
    #[must_use]
    pub fn starting() -> Self {
        use PieceKind::{Bishop, King, Knight, Queen, Rook};

        let mut position = Self {
            pieces: [None; 64],
            side_to_move: Color::White,
            castling_rights: CastlingRights::ALL,
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: NonZeroU64::MIN,
        };
        let back_rank = [Rook, Knight, Bishop, Queen, King, Bishop, Knight, Rook];
        for (file, kind) in (0..8).zip(back_rank) {
            for (color, home, pawns) in [(Color::White, 0, 1), (Color::Black, 7, 6)] {
                position.put(Square::new(file, home), Piece { color, kind });
                position.put(
                    Square::new(file, pawns),
                    Piece {
                        color,
                        kind: PieceKind::Pawn,
                    },
                );
            }
        }
        position
    }

    fn put(&mut self, square: Square, piece: Piece) {
        self.set_piece(square, Some(piece));
    }

    fn set_piece(&mut self, square: Square, piece: Option<Piece>) {
        if let Some(occupant) = self.pieces.get_mut(square.index()) {
            *occupant = piece;
        }
    }

    fn has_piece_at(&self, square: Square, color: Color, kind: PieceKind) -> bool {
        self.piece_at(square) == Some(Piece { color, kind })
    }

    fn piece_at(&self, square: Square) -> Option<Piece> {
        self.pieces.get(square.index()).copied().flatten()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Castle {
    Kingside,
    Queenside,
}

impl Castle {
    const fn rook_home(self, color: Color) -> Square {
        let file = match self {
            Self::Kingside => 7,
            Self::Queenside => 0,
        };
        Square::new(file, color.home_rank())
    }
}

impl Color {
    const fn home_rank(self) -> u8 {
        match self {
            Self::White => 0,
            Self::Black => 7,
        }
    }

    const fn king_home(self) -> Square {
        Square::new(4, self.home_rank())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CastlingRights(u8);

impl CastlingRights {
    const NONE: Self = Self(0);
    const ALL: Self = Self(0b1111);

    const fn mask(color: Color, castle: Castle) -> u8 {
        match (color, castle) {
            (Color::White, Castle::Kingside) => 1,
            (Color::White, Castle::Queenside) => 2,
            (Color::Black, Castle::Kingside) => 4,
            (Color::Black, Castle::Queenside) => 8,
        }
    }

    fn insert(&mut self, color: Color, castle: Castle) {
        self.0 |= Self::mask(color, castle);
    }

    const fn has(self, color: Color, castle: Castle) -> bool {
        self.0 & Self::mask(color, castle) != 0
    }
}
