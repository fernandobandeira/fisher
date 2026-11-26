use std::{convert::Infallible, ops::ControlFlow};

use rules::{Move, PieceKind, Position, Square};

const CHECKMATE: i32 = 100_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Score {
    Centipawns(i32),
    Mate {
        plies: u8,
        /// True when the root's side to move gives the checkmate.
        winning: bool,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchResult {
    pub best_move: Option<Move>,
    pub score: Score,
    pub nodes: u64,
    pub principal_variation: Vec<Move>,
}

/// Searches to the given depth in plies with alpha-beta pruning.
/// At depth zero the result has no best move, even if legal moves exist.
/// The score is from the root's side to move, and nodes exclude the root.
#[must_use]
pub fn minimax(position: &Position, depth: u8) -> SearchResult {
    let Ok(result) = search_depth(position, depth, &mut || Ok::<(), Infallible>(()));
    result
}

/// Searches one depth, calling `interrupted` before playing the move that reaches each node.
/// Returns `None` if the callback returns true anywhere in the depth.
#[must_use]
pub fn minimax_until_interrupted(
    position: &Position,
    depth: u8,
    mut interrupted: impl FnMut() -> bool,
) -> Option<SearchResult> {
    search_depth(position, depth, &mut || {
        (!interrupted()).then_some(()).ok_or(())
    })
    .ok()
}

fn search_depth<E>(
    position: &Position,
    depth: u8,
    before_node: &mut impl FnMut() -> Result<(), E>,
) -> Result<SearchResult, E> {
    let mut nodes = 0;
    // Reuse one buffer per ply across sibling searches.
    let mut variations: Vec<_> = (0..depth).map(|_| Vec::new()).collect();
    search(
        position,
        0,
        (-i32::MAX, i32::MAX),
        &mut nodes,
        &mut variations,
        before_node,
    )
    .map(|score| {
        let principal_variation = variations.into_iter().next().unwrap_or_default();
        SearchResult {
            best_move: principal_variation.first().copied(),
            score: mate_within_depth_or_centipawns(score, depth),
            nodes,
            principal_variation,
        }
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

fn search<E>(
    position: &Position,
    ply: u8,
    (mut alpha, beta): (i32, i32),
    nodes: &mut u64,
    variations: &mut [Vec<Move>],
    before_node: &mut impl FnMut() -> Result<(), E>,
) -> Result<i32, E> {
    let Some((best_variation, replies)) = variations.split_first_mut() else {
        return Ok(material(position));
    };
    best_variation.clear();
    let moves = position.legal_moves();
    if moves.is_empty() {
        return Ok(if position.is_in_check() {
            i32::from(ply) - CHECKMATE
        } else {
            0
        });
    }

    let mut best_score = -i32::MAX;
    let stopped = moves.into_iter().try_for_each(|movement| {
        let reply = before_node().and_then(|()| {
            *nodes += 1;
            search(
                &movement.play(),
                ply + 1,
                (-beta, -alpha),
                nodes,
                replies,
                before_node,
            )
        });
        reply.map_or_else(
            |error| ControlFlow::Break(Err(error)),
            |reply_score| {
                let score = -reply_score;
                if score >= beta {
                    // This score suffices for the cutoff. Its variation is not used.
                    return ControlFlow::Break(Ok(score));
                }
                if score > best_score {
                    best_score = score;
                    best_variation.clear();
                    best_variation.push(movement);
                    if let Some(reply) = replies.first() {
                        best_variation.extend_from_slice(reply);
                    }
                }
                alpha = alpha.max(score);
                ControlFlow::Continue(())
            },
        )
    });
    match stopped {
        ControlFlow::Break(result) => result,
        ControlFlow::Continue(()) => Ok(best_score),
    }
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

#[cfg(test)]
mod tests;
