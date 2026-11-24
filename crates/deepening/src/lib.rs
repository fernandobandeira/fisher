use std::{
    sync::{Arc, atomic::AtomicBool},
    time::{Duration, Instant},
};

use rules::{Move, Position};
use search::minimax;

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
    pub stop: Arc<AtomicBool>,
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
/// then exactly one `Ended`, and nothing after it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchReport {
    FinishedDepth(FinishedDepth),
    /// The move to send as `bestmove`: the chosen move of the last finished
    /// depth, or the first legal move when no depth finished. `None` only when
    /// the root has no legal move, after a `FinishedDepth` at depth 0.
    Ended(Option<Move>),
}

/// Reports each finished depth, then `Ended` with the move to send.
/// If interrupted before finishing a depth, the move is the first legal move.
/// A terminal root reports depth 0 followed by `Ended(None)`.
#[expect(
    clippy::needless_pass_by_value,
    reason = "The worker owns the request and the current fixed-depth search only borrows it"
)]
pub fn search(request: SearchRequest, report: &mut dyn FnMut(SearchReport)) {
    search_with_clock(&request, report, Instant::now);
}

fn search_with_clock(
    request: &SearchRequest,
    report: &mut dyn FnMut(SearchReport),
    now: impl FnOnce() -> Instant,
) {
    // TODO(fisher-5sg.4): Deepen under the limits of the go and read the stop flag.
    // TODO(fisher-5sg.10): Use the clocks and moves to go to set the time limit.
    const DEPTH: u8 = 4;
    let result = minimax(&request.position, DEPTH);
    let (depth, elapsed) = if result.chosen_move.is_some() {
        (DEPTH, now().duration_since(request.started))
    } else {
        (0, Duration::ZERO)
    };
    report(SearchReport::FinishedDepth(FinishedDepth {
        depth,
        score: result.score,
        nodes: result.nodes,
        elapsed,
        principal_variation: result.principal_variation,
    }));
    report(SearchReport::Ended(result.chosen_move));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_elapsed_time_from_the_clock() -> std::io::Result<()> {
        let origin = Instant::now();
        let request = SearchRequest {
            position: Position::starting(),
            limits: Limits::default(),
            stop: Arc::new(AtomicBool::new(false)),
            started: origin + Duration::from_micros(250),
        };
        let mut reports = Vec::new();

        search_with_clock(&request, &mut |report| reports.push(report), || {
            origin + Duration::from_micros(3250)
        });

        let [
            SearchReport::FinishedDepth(depth),
            SearchReport::Ended(chosen),
        ] = reports.as_slice()
        else {
            return Err(std::io::Error::other(format!(
                "unexpected reports: {reports:?}"
            )));
        };
        assert_eq!(depth.depth, 4);
        assert_eq!(depth.elapsed, Duration::from_millis(3));
        assert_eq!(depth.score, Score::Centipawns(0));
        // Published perft counts for plies 1 to 4, as in startpos.uci.
        assert_eq!(depth.nodes, 20 + 400 + 8_902 + 197_281);
        assert_eq!(depth.principal_variation.first().copied(), *chosen);
        Ok(())
    }
}
