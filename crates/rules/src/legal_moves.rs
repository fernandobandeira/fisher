use crate::{Castle, CastlingRights, Color, Move, Piece, PieceKind, Position, Promotion, Square};

const KNIGHT_OFFSETS: [(i8, i8); 8] = [
    (-2, -1),
    (-2, 1),
    (-1, -2),
    (-1, 2),
    (1, -2),
    (1, 2),
    (2, -1),
    (2, 1),
];
const DIAGONALS: [(i8, i8); 4] = [(-1, -1), (-1, 1), (1, -1), (1, 1)];
const ORTHOGONALS: [(i8, i8); 4] = [(-1, 0), (0, -1), (0, 1), (1, 0)];
const CASTLES: [Castle; 2] = [Castle::Kingside, Castle::Queenside];

impl Position {
    /// Lists all moves that leave the mover's king out of check.
    #[must_use]
    pub fn legal_moves(&self) -> Vec<Move> {
        let mut legal = Vec::new();
        let Some(king) = self.king_square() else {
            return legal;
        };

        let mut pseudo_legal = Vec::new();
        for from in Square::all() {
            let Some(piece) = self.mover_at(from) else {
                continue;
            };
            self.pseudo_legal_moves(from, piece, &mut pseudo_legal);
            legal.extend(
                pseudo_legal.drain(..).filter_map(|movement| {
                    self.legal_from_pseudo_legal(from, piece, movement, king)
                }),
            );
        }
        legal
    }

    /// Finds a legal move that matches the two squares and optional promotion.
    #[must_use]
    pub fn legal_move(
        &self,
        from: Square,
        to: Square,
        promotion: Option<Promotion>,
    ) -> Option<Move> {
        let piece = self.mover_at(from)?;
        let king = self.king_square()?;
        let mut moves = Vec::new();
        self.pseudo_legal_moves(from, piece, &mut moves);
        let movement = moves
            .into_iter()
            .find(|movement| movement.to == to && movement.promotion() == promotion)?;
        self.legal_from_pseudo_legal(from, piece, movement, king)
    }

    fn mover_at(&self, square: Square) -> Option<Piece> {
        self.piece_at(square)
            .filter(|piece| piece.color == self.side_to_move)
    }

    fn king_square(&self) -> Option<Square> {
        Square::all().find(|&square| self.has_piece_at(square, self.side_to_move, PieceKind::King))
    }

    fn pseudo_legal_moves(&self, from: Square, piece: Piece, moves: &mut Vec<PseudoLegalMove>) {
        match piece.kind {
            PieceKind::Pawn => self.pawn_moves(from, piece.color, moves),
            PieceKind::Knight => self.leaper_moves(from, piece.color, &KNIGHT_OFFSETS, moves),
            PieceKind::Bishop => self.slider_moves(from, piece.color, &DIAGONALS, moves),
            PieceKind::Rook => self.slider_moves(from, piece.color, &ORTHOGONALS, moves),
            PieceKind::Queen => {
                self.slider_moves(from, piece.color, &DIAGONALS, moves);
                self.slider_moves(from, piece.color, &ORTHOGONALS, moves);
            }
            PieceKind::King => {
                self.leaper_moves(from, piece.color, &DIAGONALS, moves);
                self.leaper_moves(from, piece.color, &ORTHOGONALS, moves);
                self.castling_moves(from, piece, moves);
            }
        }
    }

    fn pawn_moves(&self, from: Square, color: Color, moves: &mut Vec<PseudoLegalMove>) {
        let advance = color.pawn_advance();
        if let Some(to) = from
            .offset(0, advance)
            .filter(|&to| self.piece_at(to).is_none())
        {
            PseudoLegalMove::push_pawn_move(moves, to, color);
            if from.rank.0 == color.pawn_home_rank()
                && let Some(double) = to
                    .offset(0, advance)
                    .filter(|&to| self.piece_at(to).is_none())
            {
                moves.push(PseudoLegalMove {
                    to: double,
                    kind: MoveKind::DoublePawnAdvance(to),
                });
            }
        }
        for file in [-1, 1] {
            if let Some(to) = from.offset(file, advance) {
                match self.piece_at(to) {
                    Some(target) if target.can_be_captured_by(color) => {
                        PseudoLegalMove::push_pawn_move(moves, to, color);
                    }
                    None if self.en_passant == Some(to) => {
                        let captured = Square::new(to.file.0, from.rank.0);
                        if self.has_piece_at(captured, color.opposite(), PieceKind::Pawn) {
                            moves.push(PseudoLegalMove {
                                to,
                                kind: MoveKind::EnPassant(captured),
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn leaper_moves(
        &self,
        from: Square,
        color: Color,
        offsets: &[(i8, i8)],
        moves: &mut Vec<PseudoLegalMove>,
    ) {
        for &(file, rank) in offsets {
            if let Some(to) = from.offset(file, rank)
                && self
                    .piece_at(to)
                    .is_none_or(|piece| piece.can_be_captured_by(color))
            {
                moves.push(PseudoLegalMove::ordinary(to));
            }
        }
    }

    fn slider_moves(
        &self,
        from: Square,
        color: Color,
        directions: &[(i8, i8)],
        moves: &mut Vec<PseudoLegalMove>,
    ) {
        for &(file, rank) in directions {
            let mut square = from;
            while let Some(to) = square.offset(file, rank) {
                square = to;
                if let Some(piece) = self.piece_at(to) {
                    if piece.can_be_captured_by(color) {
                        moves.push(PseudoLegalMove::ordinary(to));
                    }
                    break;
                }
                moves.push(PseudoLegalMove::ordinary(to));
            }
        }
    }

    fn castling_moves(&self, from: Square, king: Piece, moves: &mut Vec<PseudoLegalMove>) {
        let color = king.color;
        if from != color.king_home() || self.is_attacked(from, color.opposite()) {
            return;
        }
        for castle in CASTLES {
            if self.castling_rights.has(color, castle)
                && self.has_piece_at(castle.rook_home(color), color, PieceKind::Rook)
                && castle
                    .between_files()
                    .iter()
                    .all(|&file| self.piece_at(Square::new(file, from.rank.0)).is_none())
            {
                // The king crosses the rook's destination.
                // legal_from_pseudo_legal checks the final square.
                let transit = castle.rook_destination(color);
                let mut during_castling = *self;
                during_castling.set_piece(from, None);
                during_castling.put(transit, king);
                if !during_castling.is_attacked(transit, color.opposite()) {
                    moves.push(PseudoLegalMove {
                        to: castle.king_destination(color),
                        kind: MoveKind::Castling(castle),
                    });
                }
            }
        }
    }

    fn legal_from_pseudo_legal(
        &self,
        from: Square,
        piece: Piece,
        movement: PseudoLegalMove,
        king: Square,
    ) -> Option<Move> {
        let to = movement.to;
        let position = self.play_pseudo_legal(from, piece, movement);
        let king_after_move = if piece.kind == PieceKind::King {
            to
        } else {
            king
        };
        if position.is_attacked(king_after_move, piece.color.opposite()) {
            return None;
        }
        Some(Move {
            from,
            to,
            promotion: movement.promotion(),
            position,
        })
    }

    fn play_pseudo_legal(&self, from: Square, piece: Piece, movement: PseudoLegalMove) -> Self {
        let mut next = *self;
        let captured = next.move_pieces(from, piece, movement);
        next.castling_rights
            .after_move(from, movement.to, piece, captured);
        next.advance_clocks(piece, captured);
        next.side_to_move = piece.color.opposite();
        next
    }

    fn move_pieces(
        &mut self,
        from: Square,
        piece: Piece,
        movement: PseudoLegalMove,
    ) -> Option<Piece> {
        let to = movement.to;
        let mut captured = self.piece_at(to);
        self.set_piece(from, None);
        self.en_passant = None;

        match movement.kind {
            MoveKind::DoublePawnAdvance(passed_square) => self.en_passant = Some(passed_square),
            MoveKind::EnPassant(captured_square) => {
                captured = self.piece_at(captured_square);
                self.set_piece(captured_square, None);
            }
            MoveKind::Castling(castle) => {
                self.set_piece(castle.rook_home(piece.color), None);
                self.put(
                    castle.rook_destination(piece.color),
                    Piece {
                        color: piece.color,
                        kind: PieceKind::Rook,
                    },
                );
            }
            MoveKind::Ordinary | MoveKind::Promotion(_) => {}
        }

        self.put(
            to,
            Piece {
                kind: movement
                    .promotion()
                    .map_or(piece.kind, Promotion::piece_kind),
                ..piece
            },
        );
        captured
    }

    fn advance_clocks(&mut self, piece: Piece, captured: Option<Piece>) {
        self.halfmove_clock = if captured.is_some() || piece.kind == PieceKind::Pawn {
            0
        } else {
            self.halfmove_clock.saturating_add(1)
        };
        if piece.color == Color::Black {
            self.fullmove_number = self.fullmove_number.saturating_add(1);
        }
    }

    fn is_attacked(&self, target: Square, attacker: Color) -> bool {
        let pawn_rank = -attacker.pawn_advance();
        self.attacked_at_offsets(
            target,
            attacker,
            PieceKind::Pawn,
            &[(-1, pawn_rank), (1, pawn_rank)],
        ) || self.attacked_at_offsets(target, attacker, PieceKind::Knight, &KNIGHT_OFFSETS)
            || self.attacked_at_offsets(target, attacker, PieceKind::King, &DIAGONALS)
            || self.attacked_at_offsets(target, attacker, PieceKind::King, &ORTHOGONALS)
            || self.attacked_along(target, attacker, &DIAGONALS, PieceKind::Bishop)
            || self.attacked_along(target, attacker, &ORTHOGONALS, PieceKind::Rook)
    }

    fn attacked_at_offsets(
        &self,
        target: Square,
        attacker: Color,
        kind: PieceKind,
        offsets: &[(i8, i8)],
    ) -> bool {
        offsets.iter().any(|&(file, rank)| {
            target
                .offset(file, rank)
                .is_some_and(|source| self.has_piece_at(source, attacker, kind))
        })
    }

    fn attacked_along(
        &self,
        target: Square,
        attacker: Color,
        directions: &[(i8, i8)],
        slider: PieceKind,
    ) -> bool {
        for &(file, rank) in directions {
            let mut square = target;
            while let Some(source) = square.offset(file, rank) {
                if let Some(piece) = self.piece_at(source) {
                    if piece.color == attacker
                        && (piece.kind == slider || piece.kind == PieceKind::Queen)
                    {
                        return true;
                    }
                    break;
                }
                square = source;
            }
        }
        false
    }
}

#[derive(Clone, Copy)]
struct PseudoLegalMove {
    to: Square,
    kind: MoveKind,
}

#[derive(Clone, Copy)]
enum MoveKind {
    Ordinary,
    Promotion(Promotion),
    DoublePawnAdvance(Square),
    EnPassant(Square),
    Castling(Castle),
}

impl PseudoLegalMove {
    const fn ordinary(to: Square) -> Self {
        Self {
            to,
            kind: MoveKind::Ordinary,
        }
    }

    const fn promotion(self) -> Option<Promotion> {
        if let MoveKind::Promotion(promotion) = self.kind {
            Some(promotion)
        } else {
            None
        }
    }

    fn push_pawn_move(moves: &mut Vec<Self>, to: Square, color: Color) {
        if to.rank.0 == color.opposite().home_rank() {
            for promotion in [
                Promotion::Queen,
                Promotion::Rook,
                Promotion::Bishop,
                Promotion::Knight,
            ] {
                moves.push(Self {
                    to,
                    kind: MoveKind::Promotion(promotion),
                });
            }
        } else {
            moves.push(Self::ordinary(to));
        }
    }
}

impl Promotion {
    const fn piece_kind(self) -> PieceKind {
        match self {
            Self::Knight => PieceKind::Knight,
            Self::Bishop => PieceKind::Bishop,
            Self::Rook => PieceKind::Rook,
            Self::Queen => PieceKind::Queen,
        }
    }
}

impl Piece {
    fn can_be_captured_by(self, color: Color) -> bool {
        // Accepted FENs can leave the opposing king under attack. Move generation still excludes king captures.
        self.color != color && self.kind != PieceKind::King
    }
}

impl Color {
    const fn opposite(self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
        }
    }

    const fn pawn_home_rank(self) -> u8 {
        match self {
            Self::White => 1,
            Self::Black => 6,
        }
    }

    const fn pawn_advance(self) -> i8 {
        match self {
            Self::White => 1,
            Self::Black => -1,
        }
    }
}

impl Castle {
    const fn between_files(self) -> &'static [u8] {
        match self {
            Self::Kingside => &[5, 6],
            Self::Queenside => &[1, 2, 3],
        }
    }

    const fn king_destination(self, color: Color) -> Square {
        let file = match self {
            Self::Kingside => 6,
            Self::Queenside => 2,
        };
        Square::new(file, color.home_rank())
    }

    const fn rook_destination(self, color: Color) -> Square {
        let file = match self {
            Self::Kingside => 5,
            Self::Queenside => 3,
        };
        Square::new(file, color.home_rank())
    }
}

impl CastlingRights {
    fn after_move(&mut self, from: Square, to: Square, piece: Piece, captured: Option<Piece>) {
        match piece.kind {
            PieceKind::King => {
                for castle in CASTLES {
                    self.remove(piece.color, castle);
                }
            }
            PieceKind::Rook => self.remove_rook(piece.color, from),
            PieceKind::Pawn | PieceKind::Knight | PieceKind::Bishop | PieceKind::Queen => {}
        }

        if let Some(Piece {
            kind: PieceKind::Rook,
            color,
        }) = captured
        {
            self.remove_rook(color, to);
        }
    }

    fn remove(&mut self, color: Color, castle: Castle) {
        self.0 &= !Self::mask(color, castle);
    }

    fn remove_rook(&mut self, color: Color, square: Square) {
        for castle in CASTLES {
            if square == castle.rook_home(color) {
                self.remove(color, castle);
            }
        }
    }
}
