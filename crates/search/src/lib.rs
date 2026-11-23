use rules::{Move, PieceKind, Position, Square};

const CHECKMATE: i32 = 100_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Score {
    Centipawns(i32),
    Mate { plies: u8, winning: bool },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SearchResult {
    pub chosen_move: Option<Move>,
    pub score: Score,
    pub nodes: u64,
}

/// Searches every legal move to the given depth in plies.
/// At depth zero the result has no chosen move, even if legal moves exist.
/// The score is from the starting side to move, and nodes exclude the starting position.
#[must_use]
pub fn minimax(position: &Position, depth: u8) -> SearchResult {
    let mut nodes = 0;
    let (chosen, score) = search(position, depth, 0, &mut nodes);
    let mate_plies = CHECKMATE - score.abs();
    let score = if (0..=i32::from(depth)).contains(&mate_plies)
        && let Ok(plies) = u8::try_from(mate_plies)
    {
        Score::Mate {
            plies,
            winning: score > 0,
        }
    } else {
        Score::Centipawns(score)
    };
    SearchResult {
        chosen_move: chosen,
        score,
        nodes,
    }
}

fn search(position: &Position, depth: u8, ply: u8, nodes: &mut u64) -> (Option<Move>, i32) {
    if depth == 0 {
        return (None, material(position));
    }
    let mut best_move = None;
    let mut best_score = i32::MIN;
    for movement in position.legal_moves() {
        *nodes += 1;
        let (_, reply_score) = search(&movement.play(), depth - 1, ply + 1, nodes);
        let score = -reply_score;
        if score > best_score {
            best_score = score;
            best_move = Some(movement);
        }
    }
    if best_move.is_none() {
        best_score = if position.is_in_check() {
            i32::from(ply) - CHECKMATE
        } else {
            0
        };
    }
    (best_move, best_score)
}

fn material(position: &Position) -> i32 {
    let color = position.side_to_move();
    Square::all()
        .filter_map(|square| position.piece_at(square))
        .map(|piece| {
            let value = match piece.kind() {
                PieceKind::Pawn => 100,
                PieceKind::Knight | PieceKind::Bishop => 300,
                PieceKind::Rook => 500,
                PieceKind::Queen => 900,
                PieceKind::King => 0,
            };
            if piece.color() == color {
                value
            } else {
                -value
            }
        })
        .sum()
}
