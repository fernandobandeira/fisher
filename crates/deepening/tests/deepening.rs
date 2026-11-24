use std::{
    sync::{Arc, atomic::AtomicBool},
    time::Instant,
};

use deepening::{Limits, Score, SearchRequest, search};
use rules::Position;

#[test]
fn reports_the_four_ply_search() -> std::io::Result<()> {
    let request = SearchRequest {
        position: Position::starting(),
        limits: Limits::default(),
        stop: Arc::new(AtomicBool::new(false)),
        started: Instant::now(),
    };
    let mut depths = Vec::new();
    let chosen_move = search(request, &mut |depth| depths.push(depth));
    assert_eq!(depths.len(), 1);
    let depth = depths
        .first()
        .ok_or_else(|| std::io::Error::other("expected one finished depth"))?;
    assert_eq!(depth.depth, 4);
    assert_eq!(depth.score, Score::Centipawns(0));
    // Published perft counts for plies 1 to 4, as in startpos.uci.
    assert_eq!(depth.nodes, 20 + 400 + 8_902 + 197_281);
    assert_eq!(depth.chosen_move, chosen_move);
    assert_eq!(depth.principal_variation.first().copied(), chosen_move);
    Ok(())
}
