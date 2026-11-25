use super::*;
use rules::{Move, Position};

const TIMED_GO: &str = "go movetime 50";
const ZERO_TIME_GO: &str = "go movetime 0";
const PUZZLE_DEPTH: Depth = Depth(1, -600, 30, 39, 769);
const STARTING_DEPTHS: [Depth; 2] = [Depth(1, 0, 20, 2, 10_000), Depth(2, 0, 440, 5, 88_000)];

#[derive(Clone, Copy)]
struct Depth(u8, i32, u64, u64, u64);

enum Step {
    Depth(Depth),
    BestMove { after_interrupt: bool },
}

struct TimedSearch {
    steps: mpsc::Sender<Step>,
    accepted: mpsc::Receiver<(Duration, bool)>,
    reported: mpsc::Receiver<()>,
}

impl TimedSearch {
    fn start() -> (Session, Self) {
        let (steps, instructions) = mpsc::channel();
        let (started, accepted) = mpsc::channel();
        let (sent, reported) = mpsc::channel();
        let session = Session::start(move |request, report| {
            let Some(movetime) = request.limits.movetime else {
                unreachable!("the scripted search requires movetime");
            };
            assert!(started.send((movetime, request.limits.infinite)).is_ok());
            while let Ok(step) = receive(&instructions) {
                match step {
                    Step::Depth(Depth(depth, cp, nodes, millis, _)) => {
                        report(SearchReport::FinishedDepth(FinishedDepth {
                            depth,
                            score: Score::Centipawns(cp),
                            nodes,
                            elapsed: Duration::from_millis(millis),
                            principal_variation: variation(request.position, depth),
                        }));
                        assert!(sent.send(()).is_ok());
                    }
                    Step::BestMove { after_interrupt } => {
                        if after_interrupt {
                            wait_for_interrupt(&request.interrupt);
                        }
                        report(SearchReport::BestMove(first_move(request.position)));
                        assert!(sent.send(()).is_ok());
                        return;
                    }
                }
            }
        });
        (
            session,
            Self {
                steps,
                accepted,
                reported,
            },
        )
    }

    fn accepted(&self, millis: u64, infinite: bool) -> io::Result<()> {
        assert_eq!(
            receive(&self.accepted)?,
            (Duration::from_millis(millis), infinite)
        );
        Ok(())
    }

    fn send(&self, step: Step) -> io::Result<()> {
        self.steps.send(step).map_err(io::Error::other)?;
        receive(&self.reported)
    }

    fn best_move(&self, after_interrupt: bool) -> io::Result<()> {
        self.send(Step::BestMove { after_interrupt })
    }

    fn depth(&self, session: &Session, root: Position, expected: Depth) -> io::Result<()> {
        self.send(Step::Depth(expected))?;
        let Depth(depth, cp, nodes, millis, nps) = expected;
        let pv = variation(root, depth)
            .into_iter()
            .map(move_text)
            .collect::<Vec<_>>();
        assert_eq!(
            session.reply()?,
            format!(
                "info depth {depth} multipv 1 score cp {cp} nodes {nodes} nps {nps} time {millis} pv {}\n",
                pv.join(" ")
            )
        );
        Ok(())
    }
}

fn first_move(position: Position) -> Option<Move> {
    position.legal_moves().into_iter().next()
}

fn move_text(movement: Move) -> String {
    format!("{}{}", movement.from(), movement.to())
}

fn variation(mut position: Position, depth: u8) -> Vec<Move> {
    (0..depth)
        .map(|_| {
            let Some(movement) = first_move(position) else {
                unreachable!("the scripted variation has a legal move");
            };
            position = movement.play();
            movement
        })
        .collect()
}

fn best_move_line(position: Position) -> String {
    let movement = first_move(position).map_or_else(|| "0000".to_owned(), move_text);
    format!("bestmove {movement}\n")
}

fn puzzle() -> io::Result<Position> {
    let fen = PUZZLE.trim_start_matches("position fen ");
    Position::from_fen(fen).map_err(io::Error::other)
}

fn ready(session: &Session) -> io::Result<()> {
    session.command("isready")?;
    assert_eq!(session.reply()?, "readyok\n");
    Ok(())
}

fn finish(session: Session) -> io::Result<()> {
    session.command("quit")?;
    assert_eq!(session.finish()?, Vec::<String>::new());
    Ok(())
}

fn start_puzzle() -> io::Result<(Session, TimedSearch, Position)> {
    let (session, search) = TimedSearch::start();
    let root = puzzle()?;
    session.command(PUZZLE)?;
    session.command(TIMED_GO)?;
    search.accepted(50, false)?;
    search.depth(&session, root, PUZZLE_DEPTH)?;
    Ok((session, search, root))
}

#[test]
fn movetime_reports_no_more_depths_after_the_scripted_expiry() -> io::Result<()> {
    let start = Position::starting();
    for depths in [&[][..], &STARTING_DEPTHS[..]] {
        let (session, search) = TimedSearch::start();
        session.command(TIMED_GO)?;
        search.accepted(50, false)?;
        for depth in depths {
            search.depth(&session, start, *depth)?;
        }
        ready(&session)?;
        search.best_move(false)?;
        assert_eq!(session.reply()?, best_move_line(start));
        finish(session)?;
    }
    let (session, search, root) = start_puzzle()?;
    ready(&session)?;
    search.best_move(false)?;
    assert_eq!(session.reply()?, best_move_line(root));
    finish(session)
}

#[test]
fn movetime_at_or_below_the_margin_gives_one_move() -> io::Result<()> {
    for (command, millis) in [(ZERO_TIME_GO, 0), ("go movetime 10", 10)] {
        let (session, search) = TimedSearch::start();
        session.command(command)?;
        search.accepted(millis, false)?;
        search.best_move(false)?;
        assert_eq!(session.reply()?, first_starting_move()?);
        session.command("stop")?;
        ready(&session)?;
        finish(session)?;
    }
    Ok(())
}

#[test]
fn stop_during_short_and_largest_movetime() -> io::Result<()> {
    let start = Position::starting();
    for (command, millis, depth) in [
        ("go movetime 11", 11, None),
        (
            "go movetime 18446744073709551615",
            u64::MAX,
            Some(Depth(1, 0, 20, 39, 512)),
        ),
    ] {
        let (session, search) = TimedSearch::start();
        session.command(command)?;
        search.accepted(millis, false)?;
        if let Some(depth) = depth {
            search.depth(&session, start, depth)?;
        }
        session.command("stop")?;
        search.best_move(true)?;
        assert_eq!(session.reply()?, best_move_line(start));
        ready(&session)?;
        finish(session)?;
    }
    Ok(())
}

#[test]
fn infinite_movetime_holds_its_move_until_stop() -> io::Result<()> {
    let (session, search) = TimedSearch::start();
    session.command("go infinite movetime 50")?;
    search.accepted(50, true)?;
    search.best_move(false)?;
    ready(&session)?;
    session.command("stop")?;
    assert_eq!(session.reply()?, first_starting_move()?);
    finish(session)
}

#[test]
fn stop_and_expiry_in_both_orders_send_one_move() -> io::Result<()> {
    for stop_first in [false, true] {
        let (session, search, root) = start_puzzle()?;
        if stop_first {
            session.command("stop")?;
        }
        search.best_move(stop_first)?;
        assert_eq!(session.reply()?, best_move_line(root));
        if !stop_first {
            session.command("stop")?;
        }
        ready(&session)?;
        finish(session)?;
    }
    Ok(())
}

#[test]
fn quit_and_end_of_input_in_both_orders_send_one_move() -> io::Result<()> {
    for quit in [false, true] {
        for command_first in [false, true] {
            check_exit_order(quit, command_first)?;
        }
    }
    Ok(())
}

fn check_exit_order(quit: bool, command_first: bool) -> io::Result<()> {
    let (mut session, search, root) = start_puzzle()?;
    if !command_first {
        search.best_move(false)?;
        assert_eq!(session.reply()?, best_move_line(root));
    }
    if quit {
        session.command("quit")?;
    } else {
        session.eof();
    }
    if command_first {
        search.best_move(true)?;
        assert_eq!(session.reply()?, best_move_line(root));
    }
    assert_eq!(session.finish()?, Vec::<String>::new());
    Ok(())
}

#[test]
fn replacement_position_and_go_before_expiry_are_rejected() -> io::Result<()> {
    let (session, search, root) = start_puzzle()?;
    session.command(B)?;
    session.command(ZERO_TIME_GO)?;
    assert_eq!(
        session.reply()?,
        "info string position rejected: waiting for stop\n"
    );
    assert_eq!(
        session.reply()?,
        "info string go rejected: waiting for stop\n"
    );
    search.best_move(false)?;
    assert_eq!(session.reply()?, best_move_line(root));
    session.command(ZERO_TIME_GO)?;
    search.accepted(0, false)?;
    search.best_move(false)?;
    assert_eq!(session.reply()?, best_move_line(root));
    finish(session)
}

#[test]
fn replacement_position_and_go_after_expiry_are_accepted() -> io::Result<()> {
    let (session, search, root) = start_puzzle()?;
    search.best_move(false)?;
    assert_eq!(session.reply()?, best_move_line(root));
    session.command(B)?;
    session.command(ZERO_TIME_GO)?;
    search.accepted(0, false)?;
    search.best_move(false)?;
    assert_eq!(session.reply()?, "bestmove h8h7\n");
    finish(session)
}

#[test]
fn queued_stop_replies_before_replacement_search_starts() -> io::Result<()> {
    let (session, search, root) = start_puzzle()?;
    for command in ["stop", B, ZERO_TIME_GO] {
        session.command(command)?;
    }
    search.best_move(true)?;
    search.accepted(0, false)?;
    // The old reply is already flushed when the replacement worker starts.
    assert_eq!(
        session.replies.try_recv().map_err(io::Error::other)?,
        best_move_line(root)
    );
    search.best_move(false)?;
    assert_eq!(session.reply()?, "bestmove h8h7\n");
    finish(session)
}
