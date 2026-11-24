use shakmaty::{Chess, Move, Position as _, Role};

// Greater than any material difference, even with promotions.
pub(super) const MATE_SCORE: i32 = 100_000;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct ReferenceResult {
    pub(super) best_move: Option<Move>,
    pub(super) score: i32,
    pub(super) nodes: u64,
}

/// Returns the first best move in shakmaty order, the score for the side to
/// move, and the node count excluding the starting position. Mate scores are
/// signed, with magnitude `MATE_SCORE` minus the plies to checkmate.
pub(super) fn minimax(position: &Chess, depth: u8) -> ReferenceResult {
    let (best_move, score, nodes) = search(position, depth, 0);
    ReferenceResult {
        best_move,
        score,
        nodes,
    }
}

fn search(position: &Chess, depth: u8, ply: u8) -> (Option<Move>, i32, u64) {
    if ply == depth {
        return (None, material(position), 0);
    }

    let mut best = None;
    let mut nodes = 0;
    for movement in position.legal_moves() {
        let mut child = position.clone();
        child.play_unchecked(movement);
        let (_, child_score, child_nodes) = search(&child, depth, ply + 1);
        let score = -child_score;

        nodes += 1 + child_nodes;
        if best.is_none_or(|(_, best_score)| score > best_score) {
            best = Some((movement, score));
        }
    }

    match best {
        Some((movement, score)) => (Some(movement), score, nodes),
        None if position.is_check() => (None, -MATE_SCORE + i32::from(ply), 0),
        None => (None, 0, 0),
    }
}

fn material(position: &Chess) -> i32 {
    [
        (Role::Pawn, 100),
        (Role::Knight, 300),
        (Role::Bishop, 300),
        (Role::Rook, 500),
        (Role::Queen, 900),
    ]
    .into_iter()
    .map(|(kind, value)| {
        let ours: i32 = position.our(kind).into_iter().map(|_| value).sum();
        let theirs: i32 = position.their(kind).into_iter().map(|_| value).sum();
        ours - theirs
    })
    .sum()
}
