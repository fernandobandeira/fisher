use std::{cmp::Reverse, convert::Infallible, num::NonZeroU8};

use rules::{Move, PieceKind, Position, Square};

mod move_order;
use move_order::{KillerMoves, MovePriority, ordered_moves};

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
    killers: Vec<KillerMoves>,
}

/// Searches to the given depth in plies with alpha-beta pruning.
/// At depth zero the result has no best move, even if legal moves exist.
/// The score is from the root's side to move, and nodes exclude the root.
#[must_use]
pub fn minimax(position: &Position, depth: u8) -> SearchResult {
    let Ok(result) = search_root_moves(
        position,
        depth,
        1,
        &[],
        &mut vec![KillerMoves::default(); usize::from(depth)],
        &mut || Ok::<(), Infallible>(()),
    );
    result.into()
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
    memory
        .killers
        .resize(usize::from(depth), KillerMoves::default());
    let result = search_root_moves(
        position,
        depth,
        usize::from(requested.get()),
        previous,
        &mut memory.killers,
        before_node,
    )
    .ok()?;
    memory.principal_variation.clear();
    if let Some((_, moves)) = result.variations.first() {
        memory.principal_variation.extend_from_slice(moves);
    }
    Some(result)
}

struct RootVariation {
    score: i32,
    legal_index: usize,
    moves: Vec<Move>,
}

struct SearchProgress<'a> {
    nodes: u64,
    killers: &'a mut [KillerMoves],
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
    killers: &mut [KillerMoves],
    before_node: &mut impl FnMut() -> Result<(), E>,
) -> Result<FinishedResult, E> {
    if depth == 0 {
        return Ok(FinishedResult {
            score: Score::Centipawns(material(position)),
            nodes: 0,
            variations: Vec::new(),
        });
    }

    let moves = position.legal_moves();
    let count = requested.min(moves.len());
    let mut ranked: Vec<RootVariation> = Vec::with_capacity(count);
    // Reuse one buffer per ply across sibling searches.
    let mut replies: Vec<_> = (1..depth).map(|_| Vec::new()).collect();
    let mut progress = SearchProgress { nodes: 0, killers };
    let mut ordered = ordered_moves(position, &moves, previous, KillerMoves::default()).peekable();
    let mut previous_best = ordered
        .peek()
        .and_then(|&(index, _, _, replies)| replies.map(|_| index));
    for (legal_index, movement, _, previous_reply) in ordered {
        let alpha = root_alpha(&ranked, count, legal_index, previous_best);
        let score = -search_move(
            movement,
            1,
            (-i32::MAX, -alpha),
            &mut progress,
            &mut replies,
            previous_reply.unwrap_or_default(),
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
        nodes: progress.nodes,
        variations: ranked
            .into_iter()
            .map(|RootVariation { score, moves, .. }| {
                (mate_within_depth_or_centipawns(score, depth), moves)
            })
            .collect(),
    })
}

fn root_alpha(
    ranked: &[RootVariation],
    count: usize,
    legal_index: usize,
    previous_best: Option<usize>,
) -> i32 {
    ranked
        .last()
        .filter(|_| ranked.len() == count)
        .map_or(-i32::MAX, |last| {
            // A tie can win the last place if this move comes earlier in legal
            // order. Keep a score equal to last.score inside the search window.
            let wins_tie = previous_best != Some(last.legal_index)
                && legal_index.cmp(&last.legal_index).is_lt();
            last.score - i32::from(wins_tie)
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
    (alpha, beta): (i32, i32),
    progress: &mut SearchProgress<'_>,
    variations: &mut [Vec<Move>],
    previous: &[Move],
    before_node: &mut impl FnMut() -> Result<(), E>,
) -> Result<i32, E> {
    let Some((best_variation, replies)) = variations.split_first_mut() else {
        return Ok(material(position));
    };
    best_variation.clear();
    let moves = position.legal_moves();
    if moves.is_empty() {
        return Ok(terminal_score(position, ply));
    }

    let mut best = BestReply {
        score: -i32::MAX,
        killer_index: None,
    };
    let ply_index = usize::from(ply);
    let killers = progress.killers.get(ply_index).copied().unwrap_or_default();
    let ordered = ordered_moves(position, &moves, previous, killers);
    for (index, movement, priority, previous_reply) in ordered {
        let wins_tie = best.wins_tie(alpha, index, priority);
        let reply_alpha = alpha.max(best.score) - i32::from(wins_tie);
        let score = -search_move(
            movement,
            ply + 1,
            (-beta, -reply_alpha),
            progress,
            replies,
            previous_reply.unwrap_or_default(),
            before_node,
        )?;
        if score >= beta {
            remember_cutoff(progress.killers.get_mut(ply_index), &movement, priority);
            return Ok(score);
        }
        if score > best.score || (score == best.score && wins_tie) {
            best.score = score;
            best.killer_index = priority.is_killer().then_some(index);
            best_variation.clear();
            best_variation.push(movement);
            best_variation.extend_from_slice(replies.first().map_or(&[], Vec::as_slice));
        }
    }
    Ok(best.score)
}

struct BestReply {
    score: i32,
    killer_index: Option<usize>,
}

impl BestReply {
    fn wins_tie(&self, alpha: i32, index: usize, priority: MovePriority) -> bool {
        // A killer can move ahead of an earlier quiet move during search.
        // Give the earlier move a chance to prove equality in the variation.
        self.score > alpha
            && priority.is_quiet()
            && self.killer_index.is_some_and(|killer| index < killer)
    }
}

fn remember_cutoff(killers: Option<&mut KillerMoves>, movement: &Move, priority: MovePriority) {
    // A remembered variation is searched first with a full window, so
    // its remembered moves cannot cut off.
    if priority.is_quiet()
        && let Some(killers) = killers
    {
        killers.remember(movement);
    }
}

fn search_move<E>(
    movement: Move,
    ply: u8,
    bounds: (i32, i32),
    progress: &mut SearchProgress<'_>,
    replies: &mut [Vec<Move>],
    previous: &[Move],
    before_node: &mut impl FnMut() -> Result<(), E>,
) -> Result<i32, E> {
    before_node()?;
    progress.nodes += 1;
    search(
        &movement.play(),
        ply,
        bounds,
        progress,
        replies,
        previous,
        before_node,
    )
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
