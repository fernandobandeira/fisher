#![cfg(test)]

use std::cell::{Cell, RefCell};

use super::*;

fn timed_request(position: Position, millis: u64, infinite: bool) -> SearchRequest {
    SearchRequest {
        position,
        limits: Limits {
            movetime: Some(Duration::from_millis(millis)),
            infinite,
            ..Limits::default()
        },
        interrupt: Arc::new(AtomicBool::new(false)),
        started: Instant::now(),
    }
}

#[test]
fn expired_before_depth_one_returns_first_legal_move() -> Result<(), rules::FenError> {
    let replacement = Position::from_fen("7k/8/5K2/6Q1/8/8/8/8 b - - 0 1")?;
    for (position, millis, elapsed, infinite) in [
        (Position::starting(), 50, 40, false),
        (Position::starting(), 0, 0, false),
        (Position::starting(), 10, 0, false),
        (Position::starting(), 50, 40, true),
        (puzzle()?, 0, 0, false),
        (replacement, 0, 0, false),
    ] {
        let request = timed_request(position, millis, infinite);
        let mut reports = Vec::new();
        let mut checks = 0;
        search_with_clock(
            &request,
            &mut |event| reports.push(event),
            &|| request.started + Duration::from_millis(elapsed),
            &mut || checks += 1,
            3,
        );
        assert_eq!(checks, 0);
        assert_eq!(
            reports,
            [SearchReport::BestMove(
                position.legal_moves().into_iter().next()
            )]
        );
    }
    Ok(())
}

#[test]
fn eleven_milliseconds_allows_a_node_at_zero_elapsed() {
    let position = Position::starting();
    let request = timed_request(position, 11, false);
    let mut reports = Vec::new();
    let mut checks = 0;
    search_with_clock(
        &request,
        &mut |event| reports.push(event),
        &|| request.started,
        &mut || {
            checks += 1;
            request.interrupt.store(checks == 2, Ordering::Relaxed);
        },
        3,
    );
    assert_eq!(checks, 2);
    assert_eq!(
        reports,
        [SearchReport::BestMove(
            position.legal_moves().into_iter().next()
        )]
    );
}

#[test]
fn expiry_or_interrupt_inside_depth_two_keeps_the_puzzle_move()
-> Result<(), Box<dyn std::error::Error>> {
    for interrupt_first in [false, true] {
        let request = timed_request(puzzle()?, 50, false);
        let elapsed = Cell::new(39);
        let checks = Cell::new(0);
        let readings = RefCell::new(Vec::new());
        let mut reports = Vec::new();
        search_with_clock(
            &request,
            &mut |event| reports.push(event),
            &|| {
                readings.borrow_mut().push(checks.get());
                request.started + Duration::from_millis(elapsed.get())
            },
            &mut || {
                checks.set(checks.get() + 1);
                if checks.get() == 1024 {
                    if interrupt_first {
                        request.interrupt.store(true, Ordering::Relaxed);
                    } else {
                        elapsed.set(40);
                    }
                }
            },
            3,
        );
        assert_eq!(checks.get(), 1024);
        let expected_readings = if interrupt_first {
            &[0, 30, 30][..]
        } else {
            &[0, 30, 30, 1024][..]
        };
        assert_eq!(readings.into_inner(), expected_readings);
        let [
            depth @ SearchReport::FinishedDepth(first),
            SearchReport::BestMove(best_move),
        ] = reports.as_slice()
        else {
            return Err(format!("unexpected reports: {reports:?}").into());
        };
        assert_puzzle_depths(std::slice::from_ref(depth), 1)?;
        assert_eq!(first.elapsed, Duration::from_millis(39));
        assert_eq!(move_text(*best_move).as_deref(), Some("g5g2"));
    }
    Ok(())
}

#[test]
fn expiry_inside_depth_three_keeps_the_second_variation() -> std::io::Result<()> {
    let request = timed_request(Position::starting(), 50, false);
    let elapsed = Cell::new(2);
    let mut reports = Vec::new();
    let mut checks = 0;
    search_with_clock(
        &request,
        &mut |event| {
            reports.push(event);
            elapsed.set(5);
        },
        &|| request.started + Duration::from_millis(elapsed.get()),
        &mut || {
            checks += 1;
            if checks == 1024 {
                elapsed.set(40);
            }
        },
        3,
    );
    assert_eq!(checks, 1024);
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
    assert_starting_depth(first, 1, 20, 2);
    assert_starting_depth(second, 2, 440, 5);
    assert_eq!(second.principal_variation.first().copied(), *best_move);
    Ok(())
}

#[test]
fn largest_movetime_survives_thirty_nine_milliseconds() -> std::io::Result<()> {
    let request = timed_request(Position::starting(), u64::MAX, false);
    let mut reports = Vec::new();
    let mut checks = 0;
    search_with_clock(
        &request,
        &mut |event| {
            reports.push(event);
            request.interrupt.store(true, Ordering::Relaxed);
        },
        &|| request.started + Duration::from_millis(39),
        &mut || checks += 1,
        3,
    );
    assert_eq!(checks, 20);
    let [
        SearchReport::FinishedDepth(first),
        SearchReport::BestMove(best_move),
    ] = reports.as_slice()
    else {
        return Err(std::io::Error::other(format!(
            "unexpected reports: {reports:?}"
        )));
    };
    assert_starting_depth(first, 1, 20, 39);
    assert_eq!(first.principal_variation.first().copied(), *best_move);
    Ok(())
}

fn assert_starting_depth(report: &FinishedDepth, depth: u8, nodes: u64, millis: u64) {
    assert_eq!(report.depth, depth);
    assert_eq!(report.nodes, nodes);
    assert_eq!(report.elapsed, Duration::from_millis(millis));
    assert_eq!(report.score, Score::Centipawns(0));
    assert_eq!(report.principal_variation.len(), usize::from(depth));
    let mut position = Position::starting();
    for movement in &report.principal_variation {
        assert!(
            position
                .legal_moves()
                .into_iter()
                .any(|legal| legal == *movement)
        );
        position = movement.play();
    }
}

#[test]
fn reads_elapsed_time_again_after_the_first_1024_nodes() {
    let request = timed_request(Position::starting(), 50, false);
    let mut checks = 0;
    let elapsed = Cell::new(39);
    search_with_clock(
        &request,
        &mut |_| {},
        &|| request.started + Duration::from_millis(elapsed.get()),
        &mut || {
            checks += 1;
            if checks == 1025 {
                elapsed.set(40);
            }
        },
        3,
    );
    assert_eq!(checks, 2048);
}

#[test]
fn timed_search_can_finish_more_than_four_depths() -> Result<(), rules::FenError> {
    let position = Position::from_fen("6bk/5p1p/5P1P/8/8/p1p5/P7/K7 b - - 0 1")?;
    let request = timed_request(position, u64::MAX, false);
    let interrupt = Arc::clone(&request.interrupt);
    let mut reports = Vec::new();
    search(request, &mut |event| {
        if matches!(
            &event,
            SearchReport::FinishedDepth(FinishedDepth { depth: 5, .. })
        ) {
            interrupt.store(true, Ordering::Relaxed);
        }
        reports.push(event);
    });
    assert_eq!(reports.len(), 6);
    assert!(matches!(
        reports.get(4),
        Some(SearchReport::FinishedDepth(FinishedDepth { depth: 5, .. }))
    ));
    Ok(())
}

#[test]
fn expired_movetime_reports_checkmate_before_reading_elapsed_time() -> Result<(), rules::FenError> {
    // fisher-r6y D41 and Stockfish 19: info depth 0 score mate 0.
    assert_expired_terminal_root(
        "7k/7Q/6K1/8/8/8/8/8 b - - 0 1",
        Score::Mate {
            plies: 0,
            winning: false,
        },
    )
}

#[test]
fn expired_movetime_reports_stalemate_before_reading_elapsed_time() -> Result<(), rules::FenError> {
    // fisher-r6y D41 and Stockfish 19: info depth 0 score cp 0.
    assert_expired_terminal_root("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1", Score::Centipawns(0))
}

fn assert_expired_terminal_root(fen: &str, score: Score) -> Result<(), rules::FenError> {
    let position = Position::from_fen(fen)?;
    for millis in [0, 10] {
        let request = timed_request(position, millis, false);
        let mut reports = Vec::new();
        let mut node_checks = 0;
        let elapsed_readings = Cell::new(0);
        search_with_clock(
            &request,
            &mut |event| reports.push(event),
            &|| {
                elapsed_readings.set(elapsed_readings.get() + 1);
                request.started
            },
            &mut || node_checks += 1,
            3,
        );
        assert_eq!(
            reports,
            [
                SearchReport::FinishedDepth(FinishedDepth {
                    depth: 0,
                    score,
                    nodes: 0,
                    elapsed: Duration::ZERO,
                    principal_variation: Vec::new(),
                }),
                SearchReport::BestMove(None),
            ],
        );
        assert_eq!(elapsed_readings.get(), 0);
        assert_eq!(node_checks, 0);
    }
    Ok(())
}

#[test]
fn counts_entries_until_nodes_or_movetime_refuses_entry() {
    for (node_limit, initial_elapsed, expiry_check, expected_entries) in [
        (19, 39, None, 19),
        (19, 40, None, 0),
        (1308, 39, Some(1024), 1023),
    ] {
        let mut request = timed_request(Position::starting(), 50, false);
        request.limits.nodes = Some(node_limit);
        let elapsed = Cell::new(initial_elapsed);
        let mut reports = Vec::new();
        let mut checks = 0;
        let entered = search_with_clock(
            &request,
            &mut |event| reports.push(event),
            &|| request.started + Duration::from_millis(elapsed.get()),
            &mut || {
                checks += 1;
                if expiry_check == Some(checks) {
                    elapsed.set(40);
                }
            },
            3,
        );
        assert_eq!(entered, expected_entries);
        let totals: &[u64] = if expiry_check.is_some() {
            &[20, 440]
        } else {
            &[]
        };
        assert_reports(&reports, totals, request.position);
    }
}
