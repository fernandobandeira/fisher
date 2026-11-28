use std::{cmp::Reverse, convert::Infallible, mem, num::NonZeroU8, ops::ControlFlow};

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

/// The result of one finished depth, with its requested ranked variations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FinishedResult {
    /// The best score from the root's side to move, also present when the root has no legal move.
    pub score: Score,
    /// Total nodes reached while finding these variations, excluding the root.
    pub nodes: u64,
    /// Exact scores and their variations, best first. Ties follow legal order,
    /// with the remembered root move first if it is still equal best.
    pub variations: Vec<(Score, Vec<Move>)>,
}

impl From<FinishedResult> for SearchResult {
    fn from(result: FinishedResult) -> Self {
        let principal_variation = result
            .variations
            .into_iter()
            .next()
            .map_or_else(Vec::new, |(_, moves)| moves);
        Self {
            best_move: principal_variation.first().copied(),
            score: result.score,
            nodes: result.nodes,
            principal_variation,
        }
    }
}

/// What search remembers for move order across finished depths of one go.
#[derive(Default)]
pub struct MoveOrderMemory {
    principal_variation: Vec<Move>,
}

/// Searches to the given depth in plies with alpha-beta pruning.
/// At depth zero the result has no best move, even if legal moves exist.
/// The score is from the root's side to move, and nodes exclude the root.
#[must_use]
pub fn minimax(position: &Position, depth: u8) -> SearchResult {
    let Ok(result) = search_depth(position, depth, &[], &mut || Ok::<(), Infallible>(()));
    result
}

/// Searches one depth, calling `interrupted` before playing the move that reaches each node.
/// Returns `None` if the callback returns true anywhere in the depth.
#[cfg(test)]
#[must_use]
fn minimax_until_interrupted(
    position: &Position,
    depth: u8,
    interrupted: impl FnMut() -> bool,
) -> Option<SearchResult> {
    minimax_variations_until_interrupted(
        position,
        depth,
        NonZeroU8::MIN,
        &mut MoveOrderMemory::default(),
        interrupted,
    )
    .map(SearchResult::from)
}

/// Searches one depth for up to `requested` variations.
/// At depth zero and at a root with no legal move, the result has no variations.
/// Calls `interrupted` before each node and returns `None` on the first refusal.
/// Only a finished depth replaces the principal variation in memory.
#[must_use]
pub fn minimax_variations_until_interrupted(
    position: &Position,
    depth: u8,
    requested: NonZeroU8,
    memory: &mut MoveOrderMemory,
    mut interrupted: impl FnMut() -> bool,
) -> Option<FinishedResult> {
    let before_node = &mut || (!interrupted()).then_some(()).ok_or(());
    let previous = &memory.principal_variation;
    let result = if requested == NonZeroU8::MIN || depth == 0 {
        search_depth(position, depth, previous, before_node).map(single_variation)
    } else {
        search_root_moves(
            position,
            depth,
            usize::from(requested.get()),
            previous,
            before_node,
        )
    }
    .ok()?;
    memory.principal_variation.clear();
    if let Some((_, moves)) = result.variations.first() {
        memory.principal_variation.extend_from_slice(moves);
    }
    Some(result)
}

fn single_variation(result: SearchResult) -> FinishedResult {
    FinishedResult {
        score: result.score,
        nodes: result.nodes,
        variations: result
            .best_move
            .map(|_| (result.score, result.principal_variation))
            .into_iter()
            .collect(),
    }
}

struct RootVariation {
    score: i32,
    legal_index: usize,
    moves: Vec<Move>,
}

impl RootVariation {
    fn rank_key(&self, previous_best: Option<usize>) -> (Reverse<i32>, bool, usize) {
        (
            Reverse(self.score),
            previous_best != Some(self.legal_index),
            self.legal_index,
        )
    }
}

fn search_root_moves<E>(
    position: &Position,
    depth: u8,
    requested: usize,
    previous: &[Move],
    before_node: &mut impl FnMut() -> Result<(), E>,
) -> Result<FinishedResult, E> {
    let mut moves = position.legal_moves();
    let (previous_index, mut previous_reply) = try_previous_move_first(&mut moves, previous);
    let mut previous_best = previous_index;
    let count = requested.min(moves.len());
    let mut ranked: Vec<RootVariation> = Vec::with_capacity(count);
    let mut replies: Vec<_> = (1..depth).map(|_| Vec::new()).collect();
    let mut nodes = 0;
    for (index, movement) in moves.into_iter().enumerate() {
        let legal_index = legal_root_index(index, previous_index);
        let alpha = root_alpha(&ranked, count, legal_index);
        let score = -search_move(
            movement,
            1,
            (-i32::MAX, -alpha),
            &mut nodes,
            &mut replies,
            mem::take(&mut previous_reply),
            before_node,
        )?;
        if score <= alpha {
            // A cutoff in the reply search leaves only an upper bound here.
            continue;
        }
        let mut moves = vec![movement];
        if let Some(reply) = replies.first() {
            moves.extend_from_slice(reply);
        }
        rank_root_variation(
            &mut ranked,
            count,
            &mut previous_best,
            RootVariation {
                score,
                legal_index,
                moves,
            },
        );
    }

    let score = ranked
        .first()
        .map_or_else(|| terminal_score(position, 0), |best| best.score);
    Ok(FinishedResult {
        score: mate_within_depth_or_centipawns(score, depth),
        nodes,
        variations: ranked
            .into_iter()
            .map(|RootVariation { score, moves, .. }| {
                (mate_within_depth_or_centipawns(score, depth), moves)
            })
            .collect(),
    })
}

fn legal_root_index(index: usize, previous_index: Option<usize>) -> usize {
    match previous_index {
        Some(previous) if index == 0 => previous,
        Some(previous) if index <= previous => index - 1,
        _ => index,
    }
}

fn root_alpha(ranked: &[RootVariation], count: usize, legal_index: usize) -> i32 {
    ranked
        .last()
        .filter(|_| ranked.len() == count)
        .map_or(-i32::MAX, |last| {
            // A tie can win the last place if this move comes earlier in legal
            // order. Keep a score equal to last.score inside the search window.
            last.score - i32::from(legal_index.cmp(&last.legal_index).is_lt())
        })
}

fn rank_root_variation(
    ranked: &mut Vec<RootVariation>,
    count: usize,
    previous_best: &mut Option<usize>,
    variation: RootVariation,
) {
    if ranked
        .first()
        .is_some_and(|best| variation.score > best.score)
        && previous_best.take().is_some()
    {
        // The new best takes a place, so even if the remembered move displaced
        // a tie at the last place, that tie still cannot make the ranking.
        ranked.sort_unstable_by_key(|entry| entry.rank_key(None));
    }
    let key = variation.rank_key(*previous_best);
    let rank = ranked.partition_point(|entry| entry.rank_key(*previous_best).cmp(&key).is_lt());
    ranked.insert(rank, variation);
    ranked.truncate(count);
}

fn search_depth<E>(
    position: &Position,
    depth: u8,
    previous: &[Move],
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
        previous,
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
    previous: &[Move],
    before_node: &mut impl FnMut() -> Result<(), E>,
) -> Result<i32, E> {
    let Some((best_variation, replies)) = variations.split_first_mut() else {
        return Ok(material(position));
    };
    best_variation.clear();
    let mut moves = position.legal_moves();
    if moves.is_empty() {
        return Ok(terminal_score(position, ply));
    }

    let (_, mut previous_reply) = try_previous_move_first(&mut moves, previous);
    let mut best_score = -i32::MAX;
    let stopped = moves.into_iter().try_for_each(|movement| {
        let reply = search_move(
            movement,
            ply + 1,
            (-beta, -alpha),
            nodes,
            replies,
            mem::take(&mut previous_reply),
            before_node,
        );
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

fn search_move<E>(
    movement: Move,
    ply: u8,
    bounds: (i32, i32),
    nodes: &mut u64,
    replies: &mut [Vec<Move>],
    previous: &[Move],
    before_node: &mut impl FnMut() -> Result<(), E>,
) -> Result<i32, E> {
    before_node()?;
    *nodes += 1;
    search(
        &movement.play(),
        ply,
        bounds,
        nodes,
        replies,
        previous,
        before_node,
    )
}

/// Returns the legal index and remaining variation of the move promoted to first.
fn try_previous_move_first<'a>(
    moves: &mut [Move],
    previous: &'a [Move],
) -> (Option<usize>, &'a [Move]) {
    let Some((remembered, replies)) = previous.split_first() else {
        return (None, &[]);
    };
    let index = moves.iter().position(|movement| {
        movement.from() == remembered.from()
            && movement.to() == remembered.to()
            && movement.promotion() == remembered.promotion()
    });
    let Some((index, prefix)) =
        index.and_then(|index| moves.get_mut(..=index).map(|prefix| (index, prefix)))
    else {
        return (None, &[]);
    };
    prefix.rotate_right(1);
    (Some(index), replies)
}

fn terminal_score(position: &Position, ply: u8) -> i32 {
    if position.is_in_check() {
        i32::from(ply) - CHECKMATE
    } else {
        0
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
