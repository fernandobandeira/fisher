use std::{
    sync::{Arc, atomic::AtomicBool},
    time::{Duration, Instant},
};

use rules::{Move, Position};
use search::minimax;

pub use search::Score;

/// Limits supplied together with an accepted go.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Limits {
    pub depth: Option<u64>,
    pub nodes: Option<u64>,
    pub movetime: Option<Duration>,
    pub infinite: bool,
}

/// Everything needed to search the position of an accepted go.
pub struct SearchRequest {
    pub position: Position,
    pub limits: Limits,
    pub stop: Arc<AtomicBool>,
    pub started: Instant,
}

/// One finished depth, with nodes and elapsed time totaled since the go.
pub struct FinishedDepth {
    pub depth: u8,
    pub score: Score,
    pub nodes: u64,
    pub elapsed: Duration,
    pub principal_variation: Vec<Move>,
    pub chosen_move: Option<Move>,
}

/// Reports finished depths in order, then returns the move when the work ends.
/// Without a finished depth, the returned move is the first legal move.
/// A terminal position returns no move.
#[expect(
    clippy::needless_pass_by_value,
    reason = "The worker owns the request and the current fixed-depth search only borrows its position"
)]
pub fn search(request: SearchRequest, report: &mut dyn FnMut(FinishedDepth)) -> Option<Move> {
    // TODO(fisher-5sg.4): Deepen under the limits of the go and read the stop flag.
    const DEPTH: u8 = 4;
    let result = minimax(&request.position, DEPTH);
    report(FinishedDepth {
        depth: DEPTH,
        score: result.score,
        nodes: result.nodes,
        elapsed: request.started.elapsed(),
        principal_variation: result.principal_variation,
        chosen_move: result.chosen_move,
    });
    result.chosen_move
}
