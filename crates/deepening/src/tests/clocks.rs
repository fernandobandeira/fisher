#![cfg(test)]

use std::cell::Cell;

use super::*;

fn request(position: Position, limits: Limits) -> SearchRequest {
    SearchRequest {
        position,
        limits,
        interrupt: Arc::new(AtomicBool::new(false)),
        started: Instant::now(),
    }
}

fn scheduled_search(
    request: &SearchRequest,
    start: Duration,
    schedule: &[(u64, Duration)],
    depth_cap: u8,
) -> (u64, Vec<SearchReport>) {
    let elapsed = Cell::new(start);
    let mut checks = 0;
    let mut readings = schedule.iter().peekable();
    // Observe the deadline at the boundaries specified by the controlled schedules.
    let mut reports = Vec::new();
    let entered = run_search::<1>(
        request,
        &mut |report| reports.push(report),
        &|| request.started + elapsed.get(),
        &mut || {
            checks += 1;
            if readings.peek().is_some_and(|(at, _)| *at == checks)
                && let Some((_, time)) = readings.next()
            {
                elapsed.set(*time);
            }
        },
        SearchLimits::new(request, depth_cap),
    );
    (entered, reports)
}

fn ms(millis: u64) -> Duration {
    Duration::from_millis(millis)
}

fn white_clock(millis: u64) -> Limits {
    Limits {
        wtime: Some(ms(millis)),
        ..Limits::default()
    }
}

struct ClockCase {
    color: Color,
    clocks: [Option<u64>; 2],
    increments: [Option<u64>; 2],
    moves_to_go: Option<u64>,
    time_for_move: u64,
    search_time: u64,
}

impl ClockCase {
    fn limits(&self) -> Limits {
        let [wtime, btime] = self.clocks.map(|clock| clock.map(ms));
        let [winc, binc] = self.increments.map(|increment| increment.map(ms));
        Limits {
            wtime,
            btime,
            winc,
            binc,
            movestogo: self.moves_to_go,
            ..Limits::default()
        }
    }
}

fn white(
    clock: u64,
    increment: Option<u64>,
    moves: Option<u64>,
    time: u64,
    search: u64,
) -> ClockCase {
    ClockCase {
        color: Color::White,
        clocks: [Some(clock), None],
        increments: [increment, None],
        moves_to_go: moves,
        time_for_move: time,
        search_time: search,
    }
}

fn black(
    clocks: [Option<u64>; 2],
    increments: [Option<u64>; 2],
    moves_to_go: Option<u64>,
    time_for_move: u64,
    search_time: u64,
) -> ClockCase {
    ClockCase {
        color: Color::Black,
        clocks,
        increments,
        moves_to_go,
        time_for_move,
        search_time,
    }
}

#[test]
fn time_for_the_move_uses_the_side_to_move_and_keeps_the_margin() -> Result<(), rules::FenError> {
    // D49–D51 and the worked values in expectations/uci-go-clocks/README.md.
    // UCI represents a negative clock as zero before making the request.
    let cases = [
        white(60_000, None, None, 2000, 1990),
        white(10_100, Some(100), None, 436, 426),
        white(2000, Some(50), None, 116, 106),
        white(60_000, None, Some(40), 1500, 1490),
        white(60_000, None, Some(1), 60_000, 59_990),
        white(12, Some(100_000), Some(1), 12, 2),
        white(330, None, None, 11, 1),
        white(300, None, None, 10, 0),
        white(200, None, None, 6, 0),
        white(0, None, None, 0, 0),
        black(
            [Some(5), Some(30_000)],
            [None, Some(1000)],
            None,
            2000,
            1990,
        ),
        white(60_000, None, Some(0), 2000, 1990),
        white(1531, Some(26), None, 77, 67),
        black(
            [Some(17), Some(9007)],
            [Some(77), Some(28)],
            Some(7),
            1314,
            1304,
        ),
        black(
            [Some(80_000), Some(9007)],
            [Some(9000), Some(28)],
            Some(7),
            1314,
            1304,
        ),
        black([None, Some(510)], [None, None], Some(1), 510, 500),
        black([None, Some(0)], [None, None], None, 0, 0),
        black([Some(60_000), Some(0)], [None, None], None, 0, 0),
        ClockCase {
            clocks: [Some(0), Some(60_000)],
            ..white(0, None, None, 0, 0)
        },
        ClockCase {
            clocks: [Some(1500), Some(0)],
            ..white(1500, None, None, 50, 40)
        },
        black([Some(0), Some(1830)], [None, None], None, 61, 51),
        white(1230, None, None, 41, 31),
        white(1500, None, None, 50, 40),
        black([None, Some(1500)], [None, None], None, 50, 40),
        white(3300, None, None, 110, 100),
    ];
    let black_to_move =
        Position::from_fen("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1")?;
    for case in cases {
        let position = match case.color {
            Color::White => Position::starting(),
            Color::Black => black_to_move,
        };
        check_clock_case(position, &case);
    }
    Ok(())
}

fn check_clock_case(position: Position, case: &ClockCase) {
    let request = request(position, case.limits());
    assert_eq!(
        SearchLimits::time_for_move_from_clock(&request.limits, position.side_to_move()),
        Some(ms(case.time_for_move)),
        "{:?}",
        request.limits
    );
    let time = ms(case.search_time);
    assert_eq!(
        SearchLimits::new(&request, 1).search_time_from_clock,
        Some(time)
    );
    let (entered, reports) = scheduled_search(&request, time, &[], 1);
    assert_eq!(entered, 0);
    assert_reports(&reports, &[], position);
    if case.search_time > 0 {
        let (entered, reports) =
            scheduled_search(&request, ms(case.search_time - 1), &[(2, time)], 1);
        assert_eq!(entered, 1);
        assert_reports(&reports, &[], position);
    }
}

#[test]
fn missing_clock_keeps_the_default_or_explicit_depth() {
    let position = Position::starting();
    let other_clock = Limits {
        btime: Some(ms(60_000)),
        ..Limits::default()
    };
    for limits in [
        other_clock.clone(),
        Limits {
            winc: Some(ms(1000)),
            ..Limits::default()
        },
        Limits {
            binc: Some(ms(1000)),
            ..Limits::default()
        },
        Limits {
            movestogo: Some(40),
            ..Limits::default()
        },
    ] {
        let (entered, reports) = run_with_limits(position, limits, 5);
        assert_eq!(entered, 216_365);
        assert_reports(&reports, &[20, 440, 9762, 216_365], position);
    }
    let (entered, reports) = run_with_limits(
        position,
        Limits {
            depth: Some(1),
            ..other_clock
        },
        5,
    );
    assert_eq!(entered, 20);
    assert_reports(&reports, &[20], position);
}

#[test]
fn a_healthy_clock_can_search_beyond_the_default_depth() -> Result<(), rules::FenError> {
    // The forced move from the infinite depth-cap test ends the game at ply 1.
    let position = Position::from_fen("6bk/5p1p/5P1P/8/8/p1p5/P7/K7 b - - 0 1")?;
    let (entered, reports) = run_with_limits(
        position,
        Limits {
            btime: Some(ms(510)),
            movestogo: Some(1),
            ..Limits::default()
        },
        5,
    );
    assert_eq!(entered, 5);
    assert_eq!(reports.len(), 6);
    assert!(matches!(
        reports.get(4),
        Some(SearchReport::FinishedDepth(FinishedDepth {
            depth: 5,
            nodes: 5,
            ..
        }))
    ));
    assert_eq!(
        reports.last(),
        Some(&SearchReport::BestMove(
            position.legal_moves().into_iter().next()
        ))
    );
    Ok(())
}

fn check_starting_position(
    limits: Limits,
    start: Duration,
    schedule: &[(u64, Duration)],
    entered: u64,
    reports: &[(u64, Duration)],
) {
    let position = Position::starting();
    let (actual_entries, actual_reports) =
        scheduled_search(&request(position, limits), start, schedule, 3);
    assert_eq!(actual_entries, entered);
    let totals: Vec<_> = reports.iter().map(|(nodes, _)| *nodes).collect();
    assert_reports(&actual_reports, &totals, position);
    for (report, (_, expected_time)) in actual_reports.iter().zip(reports) {
        let SearchReport::FinishedDepth(depth) = report else {
            unreachable!("expected finished depth")
        };
        assert_eq!(depth.elapsed, *expected_time);
    }
}

#[test]
fn starts_next_depth_at_equality_but_stops_one_microsecond_later() {
    let delay = Duration::from_micros(1);
    for (start, entries) in [(Duration::ZERO, 21), (delay, 20)] {
        let first_finished = ms(1) + start;
        check_starting_position(
            white_clock(1230),
            start,
            &[(20, first_finished), (22, ms(31))],
            entries,
            &[(20, first_finished)],
        );
    }
}

#[test]
fn clock_expiry_inside_depth_one_keeps_the_fallback() {
    check_starting_position(
        white_clock(1500),
        Duration::ZERO,
        &[(2, ms(39)), (3, ms(40))],
        2,
        &[],
    );
}

#[test]
fn clock_expiry_inside_depth_two_keeps_the_puzzle_move() -> Result<(), Box<dyn std::error::Error>> {
    for infinite in [false, true] {
        let request = request(
            puzzle()?,
            Limits {
                btime: Some(ms(1500)),
                infinite,
                ..Limits::default()
            },
        );
        let (entered, reports) =
            scheduled_search(&request, Duration::ZERO, &[(30, ms(1)), (1308, ms(40))], 3);
        assert_eq!(entered, 1307);
        let [
            first @ SearchReport::FinishedDepth(depth),
            SearchReport::BestMove(best_move),
        ] = reports.as_slice()
        else {
            return Err(format!("unexpected reports: {reports:?}").into());
        };
        assert_puzzle_depths(std::slice::from_ref(first), 1)?;
        assert_eq!(depth.elapsed, ms(1));
        assert_eq!(move_text(*best_move).as_deref(), Some("g5g2"));
    }
    Ok(())
}

#[test]
fn forecasts_with_the_duration_of_the_last_depth() {
    check_starting_position(
        white_clock(3300),
        Duration::ZERO,
        &[(20, ms(1)), (440, ms(4)), (442, ms(100))],
        441,
        &[(20, ms(1)), (440, ms(4))],
    );
}

#[test]
fn clock_forecast_applies_beside_each_finite_limit() {
    for other in [
        Limits {
            movetime: Some(ms(80)),
            ..Limits::default()
        },
        Limits {
            depth: Some(2),
            ..Limits::default()
        },
        Limits {
            nodes: Some(440),
            ..Limits::default()
        },
    ] {
        check_starting_position(
            Limits {
                wtime: Some(ms(1500)),
                ..other
            },
            Duration::ZERO,
            &[(20, ms(2)), (22, ms(80))],
            20,
            &[(20, ms(2))],
        );
    }
}

#[test]
fn clock_and_depth_can_each_end_the_search() {
    let limits = Limits {
        depth: Some(1),
        ..white_clock(1500)
    };
    check_starting_position(
        limits.clone(),
        Duration::ZERO,
        &[(20, ms(1))],
        20,
        &[(20, ms(1))],
    );
    check_starting_position(limits, Duration::ZERO, &[(2, ms(40))], 1, &[]);
}

#[test]
fn clock_and_nodes_can_each_end_the_search() {
    let limits = Limits {
        nodes: Some(19),
        ..white_clock(1500)
    };
    check_starting_position(limits.clone(), ms(1), &[], 19, &[]);
    check_starting_position(limits, Duration::ZERO, &[(2, ms(40))], 1, &[]);
}

#[test]
fn clock_and_movetime_can_each_end_the_search() {
    for (clock, end_first, deadline) in [(3300, 2, 40), (1230, 1, 31)] {
        check_starting_position(
            Limits {
                movetime: Some(ms(50)),
                ..white_clock(clock)
            },
            Duration::ZERO,
            &[
                (20, ms(end_first)),
                (22, ms(deadline - 1)),
                (23, ms(deadline)),
            ],
            22,
            &[(20, ms(end_first))],
        );
    }
}

#[test]
fn movetime_alone_does_not_forecast_the_next_depth() {
    check_starting_position(
        Limits {
            movetime: Some(ms(50)),
            ..Limits::default()
        },
        Duration::ZERO,
        &[(20, ms(2)), (22, ms(39)), (23, ms(40))],
        22,
        &[(20, ms(2))],
    );
}
