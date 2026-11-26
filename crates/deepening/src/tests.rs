#![cfg(test)]

use super::*;

const HANG_TIMEOUT: Duration = Duration::from_secs(30);

#[test]
fn interrupt_inside_the_depth_discards_it() -> std::io::Result<()> {
    use std::{io, sync::mpsc, thread};

    let (at_node, reached) = mpsc::channel();
    let (resume, may_resume) = mpsc::channel();
    let interrupt = Arc::new(AtomicBool::new(false));
    let request = SearchRequest {
        position: puzzle().map_err(io::Error::other)?,
        limits: Limits {
            depth: Some(2),
            ..Limits::default()
        },
        interrupt: Arc::clone(&interrupt),
        started: Instant::now(),
    };
    let expected = request.position.legal_moves().into_iter().next();
    assert_eq!(move_text(expected).as_deref(), Some("c5b4"));
    let worker = thread::spawn(move || {
        let mut reports = Vec::new();
        let mut interrupt_checks = 0;
        let entered = search_with_elapsed_time(
            &request,
            &mut |report| reports.push(report),
            &Instant::now,
            &mut || {
                interrupt_checks += 1;
                if interrupt_checks == 15 {
                    assert!(at_node.send(()).is_ok());
                    assert!(may_resume.recv_timeout(HANG_TIMEOUT).is_ok());
                }
            },
            2,
        );
        (interrupt_checks, entered, reports)
    });
    reached
        .recv_timeout(HANG_TIMEOUT)
        .map_err(io::Error::other)?;
    interrupt.store(true, Ordering::Relaxed);
    resume.send(()).map_err(io::Error::other)?;
    let (interrupt_checks, entered, reports) = worker
        .join()
        .map_err(|_| io::Error::other("search thread died"))?;
    assert_eq!(interrupt_checks, 15);
    assert_eq!(entered, 14);
    assert_eq!(reports, [SearchReport::BestMove(expected)]);
    Ok(())
}

fn puzzle() -> Result<Position, rules::FenError> {
    Position::from_fen("5r1k/pp4pp/5p2/1BbQp1r1/7K/7P/1PP3P1/3R3R b - - 3 26")
}

fn move_text(movement: Option<Move>) -> Option<String> {
    movement.map(move_name)
}

fn move_name(movement: Move) -> String {
    format!("{}{}", movement.from(), movement.to())
}

#[test]
fn reports_totals_from_the_start_of_go() -> std::io::Result<()> {
    use std::cell::Cell;

    let started = Instant::now();
    let request = SearchRequest {
        position: Position::starting(),
        limits: Limits {
            infinite: true,
            ..Limits::default()
        },
        interrupt: Arc::new(AtomicBool::new(false)),
        started,
    };
    let elapsed = Cell::new(Duration::from_millis(2));
    let mut reports = Vec::new();
    let mut node_checks = 0;
    search_with_elapsed_time(
        &request,
        &mut |report| {
            reports.push(report);
            elapsed.set(Duration::from_millis(5));
        },
        &|| started + elapsed.get(),
        &mut || node_checks += 1,
        2,
    );

    let [
        SearchReport::FinishedDepth(first),
        SearchReport::FinishedDepth(second),
        SearchReport::BestMove(best_move),
    ] = reports.as_slice()
    else {
        return Err(std::io::Error::other(format!(
            "unexpected reports: {reports:?}"
        )));
    };
    // The frozen schedule in expectations/uci-deepen-until-stop/README.md.
    assert_eq!(node_checks, 79);
    for (report, depth, nodes, millis) in [(first, 1, 20, 2), (second, 2, 79, 5)] {
        assert_eq!(report.depth, depth);
        assert_eq!(report.nodes, nodes);
        assert_eq!(report.elapsed, Duration::from_millis(millis));
        assert_eq!(report.score, Score::Centipawns(0));
    }
    assert_eq!(second.principal_variation.first().copied(), *best_move);
    Ok(())
}

#[test]
fn puzzle_keeps_the_last_finished_depth() -> Result<(), Box<dyn std::error::Error>> {
    for (interrupt_at, expected_move) in [(Some(60), "g5g2"), (None, "c5f2")] {
        let request = SearchRequest {
            position: puzzle()?,
            limits: Limits {
                depth: Some(2),
                ..Limits::default()
            },
            interrupt: Arc::new(AtomicBool::new(false)),
            started: Instant::now(),
        };
        let mut reports = Vec::new();
        let mut checks = 0;
        let entered = search_with_elapsed_time(
            &request,
            &mut |report| reports.push(report),
            &|| request.started,
            &mut || {
                checks += 1;
                if interrupt_at == Some(checks) {
                    request.interrupt.store(true, Ordering::Relaxed);
                }
            },
            3,
        );
        let Some((SearchReport::BestMove(best_move), depths)) = reports.split_last() else {
            return Err(format!("unexpected reports: {reports:?}").into());
        };
        assert_eq!(move_text(*best_move).as_deref(), Some(expected_move));
        assert_puzzle_depths(depths, if interrupt_at.is_some() { 1 } else { 2 })?;
        if let Some(interrupt_at) = interrupt_at {
            assert_eq!(checks, interrupt_at);
            assert_eq!(entered, 59);
        } else {
            let Some(SearchReport::FinishedDepth(last)) = depths.last() else {
                return Err("missing depth 2".into());
            };
            assert_eq!(checks, last.nodes);
            assert_eq!(entered, last.nodes);
        }
    }
    Ok(())
}

fn assert_puzzle_depths(
    depths: &[SearchReport],
    count: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let expected = [
        (1, 30..=30, -600, &["g5g2"][..]),
        // The lower bound of 90 comes from fact 5 in expectations/uci-cutoff/README.md.
        (2, 90..=1308, -700, &["c5f2", "g2g3"][..]),
    ];
    assert_eq!(depths.len(), count);
    for (report, (depth, nodes, score, pv)) in depths.iter().zip(expected) {
        let SearchReport::FinishedDepth(report) = report else {
            return Err(format!("unexpected depth report: {report:?}").into());
        };
        assert_eq!(report.depth, depth);
        assert!(nodes.contains(&report.nodes), "depth {depth}: {report:?}");
        assert_eq!(report.score, Score::Centipawns(score));
        let variation: Vec<_> = report
            .principal_variation
            .iter()
            .copied()
            .map(move_name)
            .collect();
        assert_eq!(variation, pv);
    }
    Ok(())
}

#[test]
fn infinite_search_reaches_the_production_depth_cap() -> Result<(), Box<dyn std::error::Error>> {
    let position = Position::from_fen("6bk/5p1p/5P1P/8/8/p1p5/P7/K7 b - - 0 1")?;
    let legal_moves = position.legal_moves();
    assert_eq!(legal_moves.len(), 1);
    let best_move = legal_moves.into_iter().next();
    assert_eq!(move_text(best_move).as_deref(), Some("c3c2"));
    let after_move = best_move.ok_or("expected c3c2")?.play();
    assert_eq!(after_move.legal_moves(), Vec::<Move>::new());

    let request = SearchRequest {
        position,
        limits: Limits {
            infinite: true,
            ..Limits::default()
        },
        interrupt: Arc::new(AtomicBool::new(false)),
        started: Instant::now(),
    };
    let mut reports = Vec::new();
    search(request, &mut |report| reports.push(report));
    let Some((last, depths)) = reports.split_last() else {
        return Err("search reported nothing".into());
    };
    assert_eq!(depths.len(), 255);
    for (depth, report) in (1..=255).zip(depths) {
        let SearchReport::FinishedDepth(report) = report else {
            return Err(format!("unexpected depth report: {report:?}").into());
        };
        assert_eq!(report.depth, depth);
        assert_eq!(report.nodes, u64::from(depth));
    }
    assert_eq!(*last, SearchReport::BestMove(best_move));
    Ok(())
}

#[test]
fn node_limit_counts_entries_even_when_a_depth_is_interrupted() {
    // Independent counts from expectations/uci-depth-and-nodes/README.md.
    for (limit, totals) in [
        (0, &[][..]),
        (19, &[][..]),
        (20, &[20][..]),
        (21, &[20][..]),
        (78, &[20][..]),
        (79, &[20, 79][..]),
    ] {
        let (entered, reports) = run_with_limits(
            Position::starting(),
            Limits {
                nodes: Some(limit),
                ..Limits::default()
            },
            3,
        );
        assert_eq!(entered, limit);
        assert_reports(&reports, totals, Position::starting());
    }
}

#[test]
fn depth_is_clamped_before_narrowing() {
    for depth in [3, 256, u64::MAX] {
        let position = Position::starting();
        let (entered, reports) = run_with_limits(
            position,
            Limits {
                depth: Some(depth),
                ..Limits::default()
            },
            2,
        );
        assert_eq!(entered, 79, "go depth {depth}");
        assert_reports(&reports, &[20, 79], position);
    }
}

#[test]
fn the_first_limit_reached_ends_search() {
    for (depth, nodes) in [(1, 79), (2, 20)] {
        let position = Position::starting();
        let (entered, reports) = run_with_limits(
            position,
            Limits {
                depth: Some(depth),
                nodes: Some(nodes),
                ..Limits::default()
            },
            3,
        );
        assert_eq!(entered, 20);
        assert_reports(&reports, &[20], position);
    }
}

#[test]
fn node_limit_before_the_first_finished_depth_uses_the_first_legal_move()
-> Result<(), rules::FenError> {
    let position = puzzle()?;
    let (entered, reports) = run_with_limits(
        position,
        Limits {
            nodes: Some(14),
            ..Limits::default()
        },
        2,
    );
    assert_eq!(entered, 14);
    assert_reports(&reports, &[], position);
    Ok(())
}

#[test]
fn zero_limits_enter_no_nodes() -> Result<(), rules::FenError> {
    let position = Position::from_fen("7k/8/5K1Q/8/8/8/8/8 b - - 0 1")?;
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
        let (entered, reports) = run_with_limits(position, limits, 2);
        assert_eq!(entered, 0);
        assert_eq!(reports.len(), 1);
        let [SearchReport::BestMove(best_move)] = reports.as_slice() else {
            unreachable!("expected one best move")
        };
        assert_eq!(move_text(*best_move).as_deref(), Some("h8g8"));
    }
    Ok(())
}

#[test]
fn zero_depth_reports_a_terminal_root() -> Result<(), rules::FenError> {
    let position = Position::from_fen("7k/6Q1/5K2/8/8/8/8/8 b - - 0 1")?;
    let (entered, reports) = run_with_limits(
        position,
        Limits {
            depth: Some(0),
            ..Limits::default()
        },
        2,
    );
    assert_eq!(entered, 0);
    assert_eq!(
        reports,
        [
            SearchReport::FinishedDepth(FinishedDepth {
                depth: 0,
                score: Score::Mate {
                    plies: 0,
                    winning: false,
                },
                nodes: 0,
                elapsed: Duration::ZERO,
                principal_variation: Vec::new(),
            }),
            SearchReport::BestMove(None),
        ]
    );
    Ok(())
}

fn run_with_limits(position: Position, limits: Limits, depth_cap: u8) -> (u64, Vec<SearchReport>) {
    let request = SearchRequest {
        position,
        limits,
        interrupt: Arc::new(AtomicBool::new(false)),
        started: Instant::now(),
    };
    let zero_limit = request.limits.nodes == Some(0) || request.limits.depth == Some(0);
    let mut reports = Vec::new();
    let entered = search_with_elapsed_time(
        &request,
        &mut |report| reports.push(report),
        &|| request.started,
        &mut || assert!(!zero_limit, "zero limit asked for node entry"),
        depth_cap,
    );
    (entered, reports)
}

fn assert_reports(reports: &[SearchReport], totals: &[u64], position: Position) {
    let Some((SearchReport::BestMove(best_move), depths)) = reports.split_last() else {
        unreachable!("expected depth reports and a best move")
    };
    assert_eq!(depths.len(), totals.len());
    let mut expected_move = position.legal_moves().into_iter().next();
    for ((depth, nodes), report) in (1..).zip(totals).zip(depths) {
        let SearchReport::FinishedDepth(report) = report else {
            unreachable!("expected a depth report")
        };
        assert_eq!(report.depth, depth);
        assert_eq!(report.nodes, *nodes);
        assert_eq!(report.score, Score::Centipawns(0));
        expected_move = report.principal_variation.first().copied();
    }
    assert_eq!(*best_move, expected_move);
}

mod clocks;
mod movetime;
