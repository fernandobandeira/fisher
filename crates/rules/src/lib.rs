use std::fmt;

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
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Position {
    pieces: [Option<Piece>; 64],
}

impl Position {
    #[must_use]
    pub fn starting() -> Self {
        use PieceKind::{Bishop, King, Knight, Queen, Rook};

        let mut position = Self { pieces: [None; 64] };
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

    /// Lists the legal moves of the starting position.
    #[must_use]
    pub fn legal_moves(&self) -> Vec<Move> {
        // TODO(fisher-l7f.6): Generate legal moves for every position.
        let mut moves = Vec::new();
        for file in 0..8 {
            let from = Square::new(file, 1);
            for to_rank in [2, 3] {
                moves.push(Move {
                    from,
                    to: Square::new(file, to_rank),
                    promotion: None,
                });
            }
        }
        for file in [1, 6] {
            let from = Square::new(file, 0);
            for file_offset in [-1, 1] {
                if let Some(to) = from.offset(file_offset, 2) {
                    moves.push(Move {
                        from,
                        to,
                        promotion: None,
                    });
                }
            }
        }
        moves
    }

    fn put(&mut self, square: Square, piece: Piece) {
        if let Some(occupant) = self.pieces.get_mut(square.index()) {
            *occupant = Some(piece);
        }
    }
}
