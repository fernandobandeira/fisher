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
/// If interrupted before finishing a depth, the move is the first legal move.
/// A finished search of a terminal root reports depth 0 then `BestMove(None)`.
#[expect(
    clippy::needless_pass_by_value,
    reason = "The worker owns the request throughout the search"
)]
pub fn search(request: SearchRequest, report: &mut dyn FnMut(SearchReport)) {
    search_with_clock(&request, report, &Instant::now, &mut || {}, u8::MAX);
}

struct SearchLimits<'a> {
    interrupt: &'a AtomicBool,
    depth_cap: u8,
    move_time: Option<MoveTime>,
}

impl<'a> SearchLimits<'a> {
    fn new(request: &'a SearchRequest, depth_cap: u8) -> Self {
        const DEFAULT_DEPTH: u8 = 4;
        let limits = &request.limits;
        let depth_cap = if limits.infinite || limits.movetime.is_some() {
            depth_cap
        } else {
            depth_cap.min(DEFAULT_DEPTH)
        };
        Self {
            interrupt: &request.interrupt,
            depth_cap,
            move_time: limits.movetime.map(MoveTime::new),
        }
    }

    fn stop_before_depth(&self, elapsed: &dyn Fn() -> Duration) -> bool {
        self.interrupt.load(Ordering::Relaxed)
            || self
                .move_time
                .as_ref()
                .is_some_and(|time| time.expired(elapsed))
    }

    fn stop_before_node(&mut self, elapsed: &dyn Fn() -> Duration) -> bool {
        self.interrupt.load(Ordering::Relaxed)
            || self
                .move_time
                .as_mut()
                .is_some_and(|time| time.expired_before_node(elapsed))
    }
}

struct MoveTime {
    limit: Duration,
    nodes_until_reading: u16,
}

impl MoveTime {
    const MARGIN: Duration = Duration::from_millis(10);
    const NODES_BETWEEN_READINGS: u16 = 1024;

    fn new(movetime: Duration) -> Self {
        Self {
            limit: movetime.saturating_sub(Self::MARGIN),
            nodes_until_reading: Self::NODES_BETWEEN_READINGS,
        }
    }

    fn expired(&self, elapsed: &dyn Fn() -> Duration) -> bool {
        elapsed() >= self.limit
    }

    fn expired_before_node(&mut self, elapsed: &dyn Fn() -> Duration) -> bool {
        self.nodes_until_reading -= 1;
        if self.nodes_until_reading != 0 {
            return false;
        }
        self.nodes_until_reading = Self::NODES_BETWEEN_READINGS;
        self.expired(elapsed)
    }
}

fn search_with_clock(
    request: &SearchRequest,
    report: &mut dyn FnMut(SearchReport),
    now: &dyn Fn() -> Instant,
    before_node: &mut dyn FnMut(),
    depth_cap: u8,
) {
    // TODO(fisher-5sg.10): Use the clocks and moves to go to set the time limit.
    let fallback = request.position.legal_moves().into_iter().next();
    if request.interrupt.load(Ordering::Relaxed) {
        report(SearchReport::BestMove(fallback));
        return;
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
        return;
    };

    let elapsed = || now().duration_since(request.started);
    let mut limits = SearchLimits::new(request, depth_cap);
    let mut nodes: u64 = 0;
    let mut best_move = fallback;
    for depth in 1..=limits.depth_cap {
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
        nodes = nodes.saturating_add(result.nodes);
        report(SearchReport::FinishedDepth(FinishedDepth {
            depth,
            score: result.score,
            nodes,
            elapsed: elapsed(),
            principal_variation: result.principal_variation,
        }));
    }
    report(SearchReport::BestMove(Some(best_move)));
}

#[cfg(test)]
mod tests;
