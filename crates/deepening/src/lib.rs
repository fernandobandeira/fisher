use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use rules::{Move, Position};
use search::minimax_until_interrupted;

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
    pub interrupt: Arc<AtomicBool>,
    pub started: Instant,
}

/// One finished depth, with nodes and elapsed time totaled since the go.
/// Depth 0 means the root has no legal move. Its info line carries no nodes,
/// time or principal variation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinishedDepth {
    pub depth: u8,
    pub score: Score,
    pub nodes: u64,
    pub elapsed: Duration,
    pub principal_variation: Vec<Move>,
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
/// A terminal root reports depth 0 then `BestMove(None)` before applying limits.
#[expect(
    clippy::needless_pass_by_value,
    reason = "The worker owns the request throughout the search"
)]
pub fn search(request: SearchRequest, report: &mut dyn FnMut(SearchReport)) {
    search_with_clock(&request, report, &Instant::now, &mut || {}, u8::MAX);
}

struct SearchLimits<'a> {
    interrupt: &'a AtomicBool,
    depth_limit: u8,
    time_for_move: Option<Duration>,
    node_limit: Option<u64>,
    nodes: u64,
}

impl<'a> SearchLimits<'a> {
    const MARGIN: Duration = Duration::from_millis(10);
    const NODES_BETWEEN_READINGS: u64 = 1024;

    fn new(request: &'a SearchRequest, depth_cap: u8) -> Self {
        let limits = &request.limits;
        Self {
            interrupt: &request.interrupt,
            depth_limit: Self::depth_limit(limits, depth_cap),
            time_for_move: limits
                .movetime
                .map(|time| time.saturating_sub(Self::MARGIN)),
            node_limit: limits.nodes,
            nodes: 0,
        }
    }

    fn depth_limit(limits: &Limits, depth_cap: u8) -> u8 {
        const DEFAULT_DEPTH: u8 = 4;
        if let Some(depth) = limits.depth {
            // Clamping to a u8 cap leaves only the low byte populated.
            let [depth, ..] = depth.min(u64::from(depth_cap)).to_le_bytes();
            depth
        } else if limits.infinite || limits.movetime.is_some() || limits.nodes.is_some() {
            depth_cap
        } else {
            depth_cap.min(DEFAULT_DEPTH)
        }
    }

    fn interrupted(&self) -> bool {
        self.interrupt.load(Ordering::Relaxed)
    }

    fn time_expired(&self, elapsed: &dyn Fn() -> Duration) -> bool {
        self.time_for_move.is_some_and(|limit| elapsed() >= limit)
    }

    fn stop_before_depth(&self, elapsed: &dyn Fn() -> Duration) -> bool {
        self.interrupted() || self.time_expired(elapsed) || self.node_limit == Some(self.nodes)
    }

    fn stop_before_node(&mut self, elapsed: &dyn Fn() -> Duration) -> bool {
        if self.interrupted()
            || (self.nodes % Self::NODES_BETWEEN_READINGS == Self::NODES_BETWEEN_READINGS - 1
                && self.time_expired(elapsed))
            || self.node_limit == Some(self.nodes)
        {
            return true;
        }
        self.nodes = self.nodes.saturating_add(1);
        false
    }
}

fn search_with_clock(
    request: &SearchRequest,
    report: &mut dyn FnMut(SearchReport),
    now: &dyn Fn() -> Instant,
    before_node: &mut dyn FnMut(),
    depth_cap: u8,
) -> u64 {
    // TODO(fisher-5sg.10): Use the clocks and moves to go to set the time limit.
    let fallback = request.position.legal_moves().into_iter().next();
    let mut limits = SearchLimits::new(request, depth_cap);
    if limits.interrupted() {
        report(SearchReport::BestMove(fallback));
        return 0;
    }
    let Some(fallback) = fallback else {
        let result = search::minimax(&request.position, 1);
        report(SearchReport::FinishedDepth(FinishedDepth {
            depth: 0,
            score: result.score,
            nodes: result.nodes,
            elapsed: Duration::ZERO,
            principal_variation: result.principal_variation,
        }));
        report(SearchReport::BestMove(None));
        return 0;
    };

    let elapsed = || now().duration_since(request.started);
    let mut best_move = fallback;
    for depth in 1..=limits.depth_limit {
        if limits.stop_before_depth(&elapsed) {
            break;
        }
        let Some(result) = minimax_until_interrupted(&request.position, depth, || {
            before_node();
            limits.stop_before_node(&elapsed)
        }) else {
            break;
        };
        best_move = result.best_move.unwrap_or(best_move);
        report(SearchReport::FinishedDepth(FinishedDepth {
            depth,
            score: result.score,
            nodes: limits.nodes,
            elapsed: elapsed(),
            principal_variation: result.principal_variation,
        }));
    }
    report(SearchReport::BestMove(Some(best_move)));
    limits.nodes
}

#[cfg(test)]
mod tests;
