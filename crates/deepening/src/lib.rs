use rules::Position;
use search::minimax;

pub use search::Score;

/// The completed search of one position.
pub struct SearchResult {
    pub depth: u8,
    pub score: Score,
    pub nodes: u64,
    pub principal_variation: Vec<rules::Move>,
    pub chosen_move: Option<rules::Move>,
}

/// Searches the position to four plies.
#[must_use]
pub fn search(position: &Position) -> SearchResult {
    const DEPTH: u8 = 4;
    let result = minimax(position, DEPTH);
    SearchResult {
        depth: DEPTH,
        score: result.score,
        nodes: result.nodes,
        principal_variation: result.principal_variation,
        chosen_move: result.chosen_move,
    }
}
