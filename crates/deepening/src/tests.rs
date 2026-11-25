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
        limits: Limits::default(),
        interrupt: Arc::clone(&interrupt),
        started: Instant::now(),
    };
    let expected = request.position.legal_moves().into_iter().next();
    assert_eq!(move_text(expected).as_deref(), Some("c5b4"));
    let worker = thread::spawn(move || {
        let mut reports = Vec::new();
        let mut interrupt_checks = 0;
        search_with_clock(
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
        (interrupt_checks, reports)
    });
    reached
        .recv_timeout(HANG_TIMEOUT)
        .map_err(io::Error::other)?;
    interrupt.store(true, Ordering::Relaxed);
    resume.send(()).map_err(io::Error::other)?;
    let (interrupt_checks, reports) = worker
        .join()
        .map_err(|_| io::Error::other("search thread died"))?;
    assert_eq!(interrupt_checks, 15);
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
    search_with_clock(
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
    assert_eq!(node_checks, 440);
    for (report, depth, nodes, millis) in [(first, 1, 20, 2), (second, 2, 440, 5)] {
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
    for (interrupt_at, expected_move, expected_checks) in
        [(Some(30 + 640), "g5g2", 670), (None, "c5f2", 1308)]
    {
        let request = SearchRequest {
            position: puzzle()?,
            limits: Limits {
                infinite: true,
                ..Limits::default()
            },
            interrupt: Arc::new(AtomicBool::new(false)),
            started: Instant::now(),
        };
        let mut reports = Vec::new();
        let mut checks = 0;
        search_with_clock(
            &request,
            &mut |report| reports.push(report),
            &|| request.started,
            &mut || {
                checks += 1;
                if interrupt_at == Some(checks) {
                    request.interrupt.store(true, Ordering::Relaxed);
                }
            },
            2,
        );
        assert_eq!(checks, expected_checks);
        let Some((SearchReport::BestMove(best_move), depths)) = reports.split_last() else {
            return Err(format!("unexpected reports: {reports:?}").into());
        };
        assert_eq!(move_text(*best_move).as_deref(), Some(expected_move));
        assert_puzzle_depths(depths, if interrupt_at.is_some() { 1 } else { 2 })?;
    }
    Ok(())
}

fn assert_puzzle_depths(
    depths: &[SearchReport],
    count: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let expected = [
        (1, 30, -600, &["g5g2"][..]),
        (2, 1308, -700, &["c5f2", "g2g3"][..]),
    ];
    assert_eq!(depths.len(), count);
    for (report, (depth, nodes, score, pv)) in depths.iter().zip(expected) {
        let SearchReport::FinishedDepth(report) = report else {
            return Err(format!("unexpected depth report: {report:?}").into());
        };
        assert_eq!(report.depth, depth);
        assert_eq!(report.nodes, nodes);
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
