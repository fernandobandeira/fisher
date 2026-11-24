use std::{
    error::Error,
    sync::{Arc, atomic::AtomicBool},
    time::Instant,
};

use deepening::{FinishedDepth, Limits, Score, SearchReport, SearchRequest, search};
use rules::Position;

fn reports_for(fen: &str) -> Result<Vec<SearchReport>, Box<dyn Error>> {
    let request = SearchRequest {
        position: Position::from_fen(fen)?,
        limits: Limits::default(),
        interrupt: Arc::new(AtomicBool::new(false)),
        started: Instant::now(),
    };
    let mut reports = Vec::new();
    deepening::search(request, &mut |report| reports.push(report));
    Ok(reports)
}

#[test]
fn reports_terminal_positions_without_searching_a_depth() -> Result<(), Box<dyn Error>> {
    // Positions and scores from expectations/uci/terminal-positions.uci.
    for (fen, score) in [
        (
            "7k/6Q1/5K2/8/8/8/8/8 b - - 0 1",
            Score::Mate {
                plies: 0,
                winning: false,
            },
        ),
        ("7k/5K2/6Q1/8/8/8/8/8 b - - 0 1", Score::Centipawns(0)),
    ] {
        let reports = reports_for(fen)?;
        // A terminal root has no legal first move for a principal variation.
        assert!(
            matches!(
                reports.as_slice(),
                [
                    SearchReport::FinishedDepth(FinishedDepth {
                        depth: 0,
                        score: actual,
                        principal_variation,
                        ..
                    }),
                    SearchReport::BestMove(None),
                ] if *actual == score && principal_variation.is_empty()
            ),
            "{fen}: {reports:?}"
        );
    }
    Ok(())
}

#[test]
fn reports_the_finished_depth_before_the_best_move() -> Result<(), Box<dyn Error>> {
    // The python-chess counts and the sole legal move are recorded in new-game.uci.
    let reports = reports_for("7k/8/5K1Q/8/8/8/8/8 b - - 0 1")?;
    let [
        SearchReport::FinishedDepth(report),
        SearchReport::BestMove(Some(best_move)),
    ] = reports.as_slice()
    else {
        return Err(format!("unexpected reports: {reports:?}").into());
    };
    assert_eq!(report.depth, 4);
    assert_eq!(report.nodes, 782);
    assert_eq!(
        report.score,
        Score::Mate {
            plies: 2,
            winning: false
        }
    );
    assert_eq!(format!("{}{}", best_move.from(), best_move.to()), "h8g8");
    assert_eq!(report.principal_variation.first(), Some(best_move));
    Ok(())
}

#[test]
fn interrupt_before_the_root_of_a_terminal_position() -> Result<(), Box<dyn std::error::Error>> {
    let request = SearchRequest {
        position: Position::from_fen("7k/6Q1/5K2/8/8/8/8/8 b - - 0 1")?,
        limits: Limits::default(),
        interrupt: Arc::new(AtomicBool::new(true)),
        started: Instant::now(),
    };
    let mut reports = Vec::new();
    search(request, &mut |report| reports.push(report));
    assert_eq!(reports, [SearchReport::BestMove(None)]);
    Ok(())
}
