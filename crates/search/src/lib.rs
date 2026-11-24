use rules::{Move, PieceKind, Position, Square};

const CHECKMATE: i32 = 100_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Score {
    Centipawns(i32),
    Mate { plies: u8, winning: bool },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchResult {
    pub chosen_move: Option<Move>,
    pub score: Score,
    pub nodes: u64,
    pub principal_variation: Vec<Move>,
}

/// Searches every legal move to the given depth in plies.
/// At depth zero the result has no chosen move, even if legal moves exist.
/// The score is from the starting side to move, and nodes exclude the starting position.
#[must_use]
#[expect(
    clippy::missing_panics_doc,
    reason = "the permission never refuses a node"
)]
pub fn minimax(position: &Position, depth: u8) -> SearchResult {
    #[expect(clippy::expect_used, reason = "the permission never refuses a node")]
    let result =
        minimax_with_permission(position, depth, &mut || true).expect("search cannot stop");
    result
}

/// Searches one depth, asking permission before playing the move that reaches each node.
/// Returns no result if permission is refused anywhere in the depth.
#[must_use]
pub fn minimax_with_permission(
    position: &Position,
    depth: u8,
    may_enter: &mut dyn FnMut() -> bool,
) -> Option<SearchResult> {
    let mut nodes = 0;
    // Reuse one buffer per ply across sibling searches.
    let mut variations: Vec<_> = (0..depth).map(|_| Vec::new()).collect();
    let score = search(position, 0, &mut nodes, &mut variations, may_enter)?;
    let principal_variation = variations.into_iter().next().unwrap_or_default();
    Some(SearchResult {
        chosen_move: principal_variation.first().copied(),
        score: mate_within_depth_or_centipawns(score, depth),
        nodes,
        principal_variation,
    })
}

/// Returns a mate score if the mate is within the search depth, or a centipawn score otherwise.
fn mate_within_depth_or_centipawns(score: i32, depth: u8) -> Score {
    let mate_plies = CHECKMATE - score.abs();
    if let Ok(plies) = u8::try_from(mate_plies)
        && plies <= depth
    {
        Score::Mate {
            plies,
            winning: score > 0,
        }
    } else {
        Score::Centipawns(score)
    }
}

fn search(
    position: &Position,
    ply: u8,
    nodes: &mut u64,
    variations: &mut [Vec<Move>],
    may_enter: &mut dyn FnMut() -> bool,
) -> Option<i32> {
    let Some((best_variation, replies)) = variations.split_first_mut() else {
        return Some(material(position));
    };
    best_variation.clear();
    let mut best_score = i32::MIN;
    for movement in position.legal_moves() {
        if !may_enter() {
            return None;
        }
        *nodes += 1;
        let score = -search(&movement.play(), ply + 1, nodes, replies, may_enter)?;
        if score > best_score {
            best_score = score;
            best_variation.clear();
            best_variation.push(movement);
            if let Some(reply) = replies.first() {
                best_variation.extend_from_slice(reply);
            }
        }
    }
    if best_variation.is_empty() {
        return Some(if position.is_in_check() {
            i32::from(ply) - CHECKMATE
        } else {
            0
        });
    }
    Some(best_score)
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
