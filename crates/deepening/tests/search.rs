use std::{
    error::Error,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

use deepening::{FinishedDepth, Limits, Score, SearchReport, SearchRequest, search};
use rules::Position;

fn reports_for(fen: &str) -> Result<Vec<SearchReport>, Box<dyn Error>> {
    Ok(reports_with_limits(
        Position::from_fen(fen)?,
        Limits::default(),
        4,
    ))
}

fn reports_with_limits(
    position: Position,
    limits: Limits,
    expected_depth: u8,
) -> Vec<SearchReport> {
    let request = SearchRequest {
        position,
        limits,
        interrupt: Arc::new(AtomicBool::new(false)),
        started: Instant::now(),
    };
    let mut reports = Vec::new();
    deepening::search(request, &mut |report| {
        if let SearchReport::FinishedDepth(depth) = &report {
            assert!(
                depth.depth <= expected_depth,
                "reported depth {}",
                depth.depth
            );
        }
        reports.push(report);
    });
    reports
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
        SearchReport::FinishedDepth(_),
        SearchReport::FinishedDepth(_),
        SearchReport::FinishedDepth(_),
        SearchReport::FinishedDepth(report),
        SearchReport::BestMove(Some(best_move)),
    ] = reports.as_slice()
    else {
        return Err(format!("unexpected reports: {reports:?}").into());
    };
    assert_eq!(report.depth, 4);
    assert_eq!(report.nodes, 857);
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

#[test]
fn infinite_search_reaches_past_the_default_depth() -> Result<(), Box<dyn Error>> {
    let interrupt = Arc::new(AtomicBool::new(false));
    let request = SearchRequest {
        position: Position::from_fen("7k/8/5K1Q/8/8/8/8/8 b - - 0 1")?,
        limits: Limits {
            infinite: true,
            ..Limits::default()
        },
        interrupt: Arc::clone(&interrupt),
        started: Instant::now(),
    };
    let mut depths = Vec::new();
    search(request, &mut |report| {
        if let SearchReport::FinishedDepth(report) = report {
            assert!(
                report.depth <= 5,
                "reported depth after interrupt: {report:?}"
            );
            depths.push(report.depth);
            if report.depth == 5 {
                interrupt.store(true, Ordering::Relaxed);
            }
        }
    });
    assert_eq!(depths, [1, 2, 3, 4, 5]);
    Ok(())
}

#[test]
fn explicit_limits_can_search_beyond_four_plies() -> Result<(), Box<dyn Error>> {
    let position = Position::from_fen("7k/8/5K1Q/8/8/8/8/8 b - - 0 1")?;
    // Counts from expectations/uci-depth-and-nodes/README.md.
    for (limits, highest_depth, totals) in [
        (
            Limits {
                depth: Some(5),
                ..Limits::default()
            },
            5,
            &[1, 23, 75, 857, 3138][..],
        ),
        (
            Limits {
                nodes: Some(3138),
                ..Limits::default()
            },
            5,
            &[1, 23, 75, 857, 3138][..],
        ),
        (
            Limits {
                nodes: Some(3137),
                ..Limits::default()
            },
            4,
            &[1, 23, 75, 857][..],
        ),
    ] {
        let reports = reports_with_limits(position, limits, highest_depth);
        let Some((SearchReport::BestMove(best_move), depths)) = reports.split_last() else {
            return Err(format!("unexpected reports: {reports:?}").into());
        };
        let actual: Vec<_> = depths
            .iter()
            .filter_map(|report| match report {
                SearchReport::FinishedDepth(report) => Some(report.nodes),
                SearchReport::BestMove(_) => None,
            })
            .collect();
        assert_eq!(actual, totals);
        assert_eq!(*best_move, position.legal_moves().into_iter().next());
    }
    Ok(())
}

#[test]
fn zero_limits_return_the_first_legal_move_or_the_terminal_score() -> Result<(), Box<dyn Error>> {
    let position = Position::from_fen("7k/8/5K1Q/8/8/8/8/8 b - - 0 1")?;
    let terminal = Position::from_fen("7k/6Q1/5K2/8/8/8/8/8 b - - 0 1")?;
    let movement = position.legal_moves().into_iter().next();
    assert!(movement.is_some());
    for limits in [
        Limits {
            depth: Some(0),
            ..Limits::default()
        },
        Limits {
            nodes: Some(0),
            ..Limits::default()
        },
    ] {
        assert_eq!(
            reports_with_limits(position, limits.clone(), 0),
            [SearchReport::BestMove(movement)]
        );
        assert_eq!(
            reports_with_limits(terminal, limits, 0),
            [
                SearchReport::FinishedDepth(FinishedDepth {
                    depth: 0,
                    score: Score::Mate {
                        plies: 0,
                        winning: false
                    },
                    nodes: 0,
                    elapsed: std::time::Duration::ZERO,
                    principal_variation: Vec::new(),
                }),
                SearchReport::BestMove(None),
            ]
        );
    }
    Ok(())
}
