#![cfg(test)]

use super::*;

#[test]
fn interrupt_inside_the_depth_discards_it() -> std::io::Result<()> {
    use std::{io, sync::mpsc, thread};

    let (at_node, reached) = mpsc::channel();
    let (resume, may_resume) = mpsc::channel();
    let interrupt = Arc::new(AtomicBool::new(false));
    let request = SearchRequest {
        position: Position::starting(),
        limits: Limits::default(),
        interrupt: Arc::clone(&interrupt),
        started: Instant::now(),
    };
    let expected = request.position.legal_moves().into_iter().next();
    let worker = thread::spawn(move || {
        let mut reports = Vec::new();
        let mut interrupt_checks = 0;
        search_with_clock(
            &request,
            &mut |report| reports.push(report),
            &Instant::now,
            &mut || {
                interrupt_checks += 1;
                if interrupt_checks == 3 {
                    assert!(at_node.send(()).is_ok());
                    assert!(may_resume.recv_timeout(Duration::from_secs(5)).is_ok());
                }
            },
        );
        (interrupt_checks, reports)
    });
    reached
        .recv_timeout(Duration::from_secs(5))
        .map_err(io::Error::other)?;
    interrupt.store(true, Ordering::Relaxed);
    resume.send(()).map_err(io::Error::other)?;
    let (interrupt_checks, reports) = worker
        .join()
        .map_err(|_| io::Error::other("search thread died"))?;
    assert_eq!(interrupt_checks, 3);
    assert_eq!(reports, [SearchReport::BestMove(expected)]);
    Ok(())
}

#[test]
fn reports_elapsed_time_from_the_clock() -> std::io::Result<()> {
    let origin = Instant::now();
    let request = SearchRequest {
        position: Position::starting(),
        limits: Limits::default(),
        interrupt: Arc::new(AtomicBool::new(false)),
        started: origin + Duration::from_micros(250),
    };
    let mut reports = Vec::new();

    search_with_clock(
        &request,
        &mut |report| reports.push(report),
        &|| origin + Duration::from_micros(3250),
        &mut || {},
    );

    let [
        SearchReport::FinishedDepth(depth),
        SearchReport::BestMove(best_move),
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
    assert_eq!(depth.principal_variation.first().copied(), *best_move);
    Ok(())
}
