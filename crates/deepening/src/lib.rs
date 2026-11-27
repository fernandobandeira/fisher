use std::{
    num::NonZeroU8,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use rules::{Color, Move, Position};
use search::minimax_variations_until_interrupted;

pub use search::Score;

/// Limits supplied together with an accepted go.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Limits {
    pub depth: Option<u64>,
    pub nodes: Option<u64>,
    pub movetime: Option<Duration>,
    pub infinite: bool,
    pub wtime: Option<Duration>,
    pub btime: Option<Duration>,
    pub winc: Option<Duration>,
    pub binc: Option<Duration>,
    pub movestogo: Option<u64>,
}

/// Everything needed to search the position of an accepted go.
#[derive(Debug)]
pub struct SearchRequest {
    pub position: Position,
    pub limits: Limits,
    pub requested_variations: NonZeroU8,
    pub interrupt: Arc<AtomicBool>,
    pub started: Instant,
}

/// One finished depth, with nodes and elapsed time totaled since the go.
/// Depth 0 means the root has no legal move. Its info line carries no nodes,
/// time or principal variation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinishedDepth {
    pub depth: u8,
    pub nodes: u64,
    pub elapsed: Duration,
    /// The root score at depth 0. A searched depth has no terminal score.
    pub terminal_score: Option<Score>,
    /// Exact scores and their variations, best first. Empty only at depth 0.
    pub variations: Vec<(Score, Vec<Move>)>,
}

/// Reports from the work of a search, in the order the session consumes them.
///
/// A search sends zero or more `FinishedDepth` reports in increasing depth,
/// then exactly one `BestMove`, and nothing after it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchReport {
    FinishedDepth(FinishedDepth),
    /// The move to send as `bestmove`: the best move of the last finished
    /// depth, or the first legal move when no depth finished. `None` only when
    /// the root has no legal move.
    BestMove(Option<Move>),
}

/// Reports each finished depth, then `BestMove` with the move to send.
/// If interrupted or limited before depth 1 finishes, the move is the first
/// legal move. A zero depth, node or time limit permits no searched depth.
/// A terminal root reports depth 0 then `BestMove(None)`, even if interrupted.
#[expect(
    clippy::needless_pass_by_value,
    reason = "The worker owns the request throughout the search"
)]
pub fn search(request: SearchRequest, report: &mut dyn FnMut(SearchReport)) {
    search_with_elapsed_time(&request, report, &Instant::now, &mut || {}, u8::MAX);
}

struct SearchLimits<'a> {
    interrupt: &'a AtomicBool,
    depth_limit: u8,
    search_time: Option<Duration>,
    search_time_from_clock: Option<Duration>,
    node_limit: Option<u64>,
    nodes: u64,
}

impl<'a> SearchLimits<'a> {
    const MARGIN: Duration = Duration::from_millis(10);
    const NODES_BETWEEN_READINGS: u64 = 1024;
    const DEPTH_GROWTH_FACTOR: u32 = 15;

    fn new(request: &'a SearchRequest, depth_cap: u8) -> Self {
        let limits = &request.limits;
        let time_for_move_from_clock =
            Self::time_for_move_from_clock(limits, request.position.side_to_move());
        Self {
            interrupt: &request.interrupt,
            depth_limit: Self::depth_limit(limits, depth_cap, time_for_move_from_clock.is_some()),
            search_time: limits
                .movetime
                .into_iter()
                .chain(time_for_move_from_clock)
                .min()
                .map(|time| time.saturating_sub(Self::MARGIN)),
            search_time_from_clock: time_for_move_from_clock
                .map(|time| time.saturating_sub(Self::MARGIN)),
            node_limit: limits.nodes,
            nodes: 0,
        }
    }

    fn time_for_move_from_clock(limits: &Limits, side_to_move: Color) -> Option<Duration> {
        const DEFAULT_MOVES_TO_GO: u64 = 30;
        let (clock, increment) = match side_to_move {
            Color::White => (limits.wtime, limits.winc),
            Color::Black => (limits.btime, limits.binc),
        };
        let moves_to_go = limits
            .movestogo
            .filter(|&moves| moves > 0)
            .unwrap_or(DEFAULT_MOVES_TO_GO);
        clock.map(|clock| {
            // Divide in whole milliseconds before adding the increment.
            let millis = clock.as_millis() / u128::from(moves_to_go);
            let whole_millis = Duration::new(
                u64::try_from(millis / 1000).unwrap_or(u64::MAX),
                u32::try_from(millis % 1000).unwrap_or_default() * 1_000_000,
            );
            whole_millis
                .saturating_add(increment.unwrap_or_default())
                .min(clock)
        })
    }

    fn depth_limit(limits: &Limits, depth_cap: u8, has_clock: bool) -> u8 {
        const DEFAULT_DEPTH: u8 = 4;
        if let Some(depth) = limits.depth {
            // Clamping to a u8 cap leaves only the low byte populated.
            let [depth, ..] = depth.min(u64::from(depth_cap)).to_le_bytes();
            depth
        } else if has_clock
            || limits.infinite
            || limits.movetime.is_some()
            || limits.nodes.is_some()
        {
            depth_cap
        } else {
            depth_cap.min(DEFAULT_DEPTH)
        }
    }

    fn interrupted(&self) -> bool {
        self.interrupt.load(Ordering::Relaxed)
    }

    fn start_depth(
        &self,
        elapsed: &dyn Fn() -> Duration,
        last_depth_duration: Option<Duration>,
    ) -> Option<Duration> {
        if self.interrupted() {
            return None;
        }
        let started = self.search_time.map_or(Duration::ZERO, |_| elapsed());
        if self.search_time.is_some_and(|limit| started >= limit) {
            return None;
        }
        let forecast = last_depth_duration
            .unwrap_or_default()
            .saturating_mul(Self::DEPTH_GROWTH_FACTOR);
        if self
            .search_time_from_clock
            .is_some_and(|time| forecast > time.saturating_sub(started))
        {
            return None;
        }
        (self.node_limit != Some(self.nodes)).then_some(started)
    }

    fn stop_before_node<const NODES_BETWEEN_READINGS: u64>(
        &mut self,
        elapsed: &dyn Fn() -> Duration,
    ) -> bool {
        if self.interrupted()
            || (self.nodes % NODES_BETWEEN_READINGS == NODES_BETWEEN_READINGS - 1
                && self.search_time.is_some_and(|limit| elapsed() >= limit))
            || self.node_limit == Some(self.nodes)
        {
            return true;
        }
        self.nodes = self.nodes.saturating_add(1);
        false
    }
}

fn search_with_elapsed_time(
    request: &SearchRequest,
    report: &mut dyn FnMut(SearchReport),
    now: &dyn Fn() -> Instant,
    before_node: &mut dyn FnMut(),
    depth_cap: u8,
) -> u64 {
    run_search::<{ SearchLimits::NODES_BETWEEN_READINGS }>(
        request,
        report,
        now,
        before_node,
        SearchLimits::new(request, depth_cap),
    )
}

fn run_search<const NODES_BETWEEN_READINGS: u64>(
    request: &SearchRequest,
    report: &mut dyn FnMut(SearchReport),
    now: &dyn Fn() -> Instant,
    before_node: &mut dyn FnMut(),
    mut limits: SearchLimits<'_>,
) -> u64 {
    let Some(fallback) = request.position.legal_moves().into_iter().next() else {
        let result = search::minimax(&request.position, 1);
        report(SearchReport::FinishedDepth(FinishedDepth {
            depth: 0,
            nodes: result.nodes,
            elapsed: Duration::ZERO,
            terminal_score: Some(result.score),
            variations: Vec::new(),
        }));
        report(SearchReport::BestMove(None));
        return 0;
    };

    let elapsed = || now().duration_since(request.started);
    let mut best_move = fallback;
    let mut last_depth_duration = None;
    for depth in 1..=limits.depth_limit {
        let Some(started) = limits.start_depth(&elapsed, last_depth_duration) else {
            break;
        };
        let Some(result) = minimax_variations_until_interrupted(
            &request.position,
            depth,
            request.requested_variations,
            || {
                before_node();
                limits.stop_before_node::<NODES_BETWEEN_READINGS>(&elapsed)
            },
        ) else {
            break;
        };
        let Some(root_move) = result
            .variations
            .first()
            .and_then(|(_, moves)| moves.first())
        else {
            unreachable!("a finished search of a nonterminal root has a principal variation");
        };
        best_move = *root_move;
        let finished = elapsed();
        last_depth_duration = Some(finished.saturating_sub(started));
        report(SearchReport::FinishedDepth(FinishedDepth {
            depth,
            nodes: limits.nodes,
            elapsed: finished,
            terminal_score: None,
            variations: result.variations,
        }));
    }
    report(SearchReport::BestMove(Some(best_move)));
    limits.nodes
}

#[cfg(test)]
mod tests;
