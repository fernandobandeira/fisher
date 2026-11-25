// The UCI observations in expectations/uci-go-clocks/README.md.
// Deepening tests control time and count entries. This seam delivers the
// corresponding reports and observes how the session handles them.

use super::*;

const AT_ONE_MS: Depth = Depth(1, 0, 20, 1, 20_000);
const AT_TWO_MS: Depth = Depth(1, 0, 20, 2, 10_000);
const AT_FOUR_MS: Depth = Depth(2, 0, 440, 4, 110_000);
const CLOCK_PUZZLE_DEPTH: Depth = Depth(1, -600, 30, 1, 30_000);

fn white_clock(millis: u64) -> Limits {
    Limits {
        wtime: Some(Duration::from_millis(millis)),
        ..Limits::default()
    }
}

fn finish_finite_search(session: Session, search: &TimedSearch, best: &str) -> io::Result<()> {
    search.best_move(false)?;
    assert_eq!(session.reply()?, best);
    ready(&session)?;
    finish(session)
}

#[test]
fn passes_the_clocks_increments_and_moves_to_go() -> io::Result<()> {
    let (session, search) = TimedSearch::start();
    session.command(A)?;
    session.command("go wtime 17 winc 77 btime 9007 binc 28 movestogo 7")?;
    search.accepted_limits(&Limits {
        wtime: Some(Duration::from_millis(17)),
        winc: Some(Duration::from_millis(77)),
        btime: Some(Duration::from_millis(9007)),
        binc: Some(Duration::from_millis(28)),
        movestogo: Some(7),
        ..Limits::default()
    })?;
    finish_finite_search(session, &search, "bestmove h8g8\n")
}

#[test]
fn forwards_an_exhausted_clock_as_zero_even_with_a_healthy_other_clock() -> io::Result<()> {
    for command in ["go wtime 0", "go wtime -500"] {
        check_starting_schedule(command, &white_clock(0), &[])?;
    }
    for command in ["go wtime 0 btime 60000", "go wtime -500 btime 60000"] {
        check_starting_schedule(
            command,
            &Limits {
                btime: Some(Duration::from_millis(60_000)),
                ..white_clock(0)
            },
            &[],
        )?;
    }
    for (commands, white_time) in [
        (["go btime 0", "go btime -500"], None),
        (
            ["go wtime 60000 btime 0", "go wtime 60000 btime -500"],
            Some(Duration::from_millis(60_000)),
        ),
    ] {
        for command in commands {
            let (session, search) = TimedSearch::start();
            session.command(A)?;
            session.command(command)?;
            search.accepted_limits(&Limits {
                wtime: white_time,
                btime: Some(Duration::from_millis(0)),
                ..Limits::default()
            })?;
            finish_finite_search(session, &search, "bestmove h8g8\n")?;
        }
    }
    Ok(())
}

#[test]
fn only_negative_decimal_clocks_are_read_as_zero() -> io::Result<()> {
    for command in [
        "go wtime - btime 60000",
        "go wtime -later btime 60000",
        "go winc -500 btime 60000",
    ] {
        let (session, search) = TimedSearch::start();
        session.command(command)?;
        search.accepted_limits(&Limits {
            btime: Some(Duration::from_millis(60_000)),
            ..Limits::default()
        })?;
        finish_finite_search(session, &search, &best_move_line(Position::starting()))?;
    }
    Ok(())
}

#[test]
fn forwards_a_healthy_clock_beside_an_exhausted_other_clock() -> io::Result<()> {
    for command in ["go wtime 1500 btime 0", "go wtime 1500 btime -500"] {
        check_starting_schedule(
            command,
            &Limits {
                btime: Some(Duration::from_millis(0)),
                ..white_clock(1500)
            },
            &[AT_TWO_MS],
        )?;
    }
    for command in ["go wtime 0 btime 1830", "go wtime -500 btime 1830"] {
        let (session, search) = TimedSearch::start();
        session.command(A)?;
        session.command(command)?;
        search.accepted_limits(&Limits {
            btime: Some(Duration::from_millis(1830)),
            ..white_clock(0)
        })?;
        let root =
            Position::from_fen(A.trim_start_matches("position fen ")).map_err(io::Error::other)?;
        search.depth(&session, root, Depth(1, -900, 1, 1, 1000))?;
        finish_finite_search(session, &search, "bestmove h8g8\n")?;
    }
    Ok(())
}

fn check_starting_schedule(
    command: &'static str,
    limits: &Limits,
    depths: &[Depth],
) -> io::Result<()> {
    let (session, search) = TimedSearch::start();
    let root = Position::starting();
    session.command(command)?;
    search.accepted_limits(limits)?;
    for &depth in depths {
        search.depth(&session, root, depth)?;
    }
    finish_finite_search(session, &search, &best_move_line(root))
}

#[test]
fn clock_reports_only_the_finished_depths() -> io::Result<()> {
    // Equality and the 1 µs dispatch delay both display 1 ms and one depth.
    check_starting_schedule("go wtime 1230", &white_clock(1230), &[AT_ONE_MS])?;
    check_starting_schedule("go wtime 1500", &white_clock(1500), &[])?;
    check_starting_schedule(
        "go wtime 3300",
        &white_clock(3300),
        &[AT_ONE_MS, AT_FOUR_MS],
    )?;
    check_puzzle(false)
}

#[test]
fn missing_clock_preserves_the_other_limits_and_default() -> io::Result<()> {
    let starting_depths = [
        AT_ONE_MS,
        Depth(2, 0, 440, 1, 440_000),
        Depth(3, 0, 9762, 1, 9_762_000),
        Depth(4, 0, 216_365, 1, 216_365_000),
    ];
    let other_clock = Limits {
        btime: Some(Duration::from_millis(60_000)),
        ..Limits::default()
    };
    for (command, limits) in [
        ("go btime 60000", other_clock.clone()),
        (
            "go winc 1000",
            Limits {
                winc: Some(Duration::from_millis(1000)),
                ..Limits::default()
            },
        ),
        (
            "go binc 1000",
            Limits {
                binc: Some(Duration::from_millis(1000)),
                ..Limits::default()
            },
        ),
        (
            "go movestogo 40",
            Limits {
                movestogo: Some(40),
                ..Limits::default()
            },
        ),
    ] {
        check_starting_schedule(command, &limits, &starting_depths)?;
    }
    check_starting_schedule(
        "go btime 60000 depth 1",
        &Limits {
            depth: Some(1),
            ..other_clock
        },
        &[AT_ONE_MS],
    )
}

#[test]
fn clock_and_other_finite_limits_pass_through_in_both_orders() -> io::Result<()> {
    for (commands, limits, depths) in [
        (
            ["go wtime 1500 movetime 80", "go movetime 80 wtime 1500"],
            Limits {
                movetime: Some(Duration::from_millis(80)),
                ..white_clock(1500)
            },
            &[AT_TWO_MS][..],
        ),
        (
            ["go wtime 1500 depth 2", "go depth 2 wtime 1500"],
            Limits {
                depth: Some(2),
                ..white_clock(1500)
            },
            &[AT_TWO_MS],
        ),
        (
            ["go wtime 1500 nodes 440", "go nodes 440 wtime 1500"],
            Limits {
                nodes: Some(440),
                ..white_clock(1500)
            },
            &[AT_TWO_MS],
        ),
        (
            ["go wtime 1500 depth 1", "go depth 1 wtime 1500"],
            Limits {
                depth: Some(1),
                ..white_clock(1500)
            },
            &[AT_ONE_MS],
        ),
        (
            ["go wtime 1500 depth 1", "go depth 1 wtime 1500"],
            Limits {
                depth: Some(1),
                ..white_clock(1500)
            },
            &[],
        ),
        // Either node or clock expiry produces only the final report.
        (
            ["go wtime 1500 nodes 19", "go nodes 19 wtime 1500"],
            Limits {
                nodes: Some(19),
                ..white_clock(1500)
            },
            &[],
        ),
        (
            ["go wtime 3300 movetime 50", "go movetime 50 wtime 3300"],
            Limits {
                movetime: Some(Duration::from_millis(50)),
                ..white_clock(3300)
            },
            &[AT_TWO_MS],
        ),
        (
            ["go wtime 1230 movetime 50", "go movetime 50 wtime 1230"],
            Limits {
                movetime: Some(Duration::from_millis(50)),
                ..white_clock(1230)
            },
            &[AT_ONE_MS],
        ),
    ] {
        for command in commands {
            check_starting_schedule(command, &limits, depths)?;
        }
    }
    Ok(())
}

fn check_puzzle(infinite: bool) -> io::Result<()> {
    let (session, search) = TimedSearch::start();
    let root = puzzle()?;
    session.command(PUZZLE)?;
    session.command(if infinite {
        "go infinite btime 1500"
    } else {
        "go btime 1500"
    })?;
    search.accepted_limits(&Limits {
        btime: Some(Duration::from_millis(1500)),
        infinite,
        ..Limits::default()
    })?;
    let best = root
        .legal_moves()
        .into_iter()
        .find(|&movement| move_text(movement) == "g5g2")
        .ok_or_else(|| io::Error::other("g5g2 must be legal"))?;
    search.depth_with_variation(&session, CLOCK_PUZZLE_DEPTH, vec![best])?;
    search.best_move(false)?;
    if infinite {
        ready(&session)?;
        session.command("stop")?;
    }
    assert_eq!(session.reply()?, "bestmove g5g2\n");
    ready(&session)?;
    finish(session)
}

#[test]
fn infinite_holds_the_move_after_clock_expiry() -> io::Result<()> {
    check_puzzle(true)
}

#[test]
fn movetime_alone_can_still_be_working_after_one_finished_depth() -> io::Result<()> {
    let (session, search) = TimedSearch::start();
    let root = Position::starting();
    session.command(TIMED_GO)?;
    search.accepted(50, false)?;
    search.depth(&session, root, AT_TWO_MS)?;
    ready(&session)?;
    finish_finite_search(session, &search, &best_move_line(root))
}
