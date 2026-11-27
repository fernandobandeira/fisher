#[path = "support/search_history.rs"]
mod search_history;

use std::{
    io::{self, Write},
    sync::{Arc, Mutex, MutexGuard, atomic::Ordering, mpsc},
    thread,
    time::{Duration, Instant},
};

use deepening::{FinishedDepth, Score, SearchReport, SearchRequest};
use search_history::{Event, HANG_TIMEOUT, ScriptedInput, check, scripted_input};

const A: &str = "position fen 7k/8/5K1Q/8/8/8/8/8 b - - 0 1";
const B: &str = "position fen 7k/8/5K2/6Q1/8/8/8/8 b - - 0 1";
const PUZZLE: &str = "position fen 5r1k/pp4pp/5p2/1BbQp1r1/7K/7P/1PP3P1/3R3R b - - 3 26";

enum Recorded {
    Command(&'static str),
    EndOfInput,
    Report(SearchReport),
    Response(String),
}

impl Recorded {
    fn as_event(&self) -> Event<'_> {
        match self {
            Self::Command(command) => Event::Command(command),
            Self::EndOfInput => Event::EndOfInput,
            Self::Report(report) => Event::Report(report.clone()),
            Self::Response(line) => Event::Response(line),
        }
    }
}

type History = Arc<Mutex<Vec<Recorded>>>;

fn recorded(history: &History) -> MutexGuard<'_, Vec<Recorded>> {
    #[expect(
        clippy::unwrap_used,
        reason = "a poisoned test recorder is a test failure"
    )]
    history.lock().unwrap()
}

struct Replies {
    history: History,
    lines: mpsc::Sender<String>,
    pending: Vec<u8>,
}

impl Write for Replies {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.pending.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        let line =
            String::from_utf8(std::mem::take(&mut self.pending)).map_err(io::Error::other)?;
        recorded(&self.history).push(Recorded::Response(line.clone()));
        self.lines.send(line).map_err(io::Error::other)
    }
}

struct Session {
    history: History,
    input: Option<ScriptedInput>,
    replies: mpsc::Receiver<String>,
    result: mpsc::Receiver<io::Result<()>>,
}

impl Session {
    fn start(
        mut search: impl FnMut(SearchRequest, &mut dyn FnMut(SearchReport)) + Send + 'static,
    ) -> Self {
        let (input, commands) = scripted_input();
        let history = History::default();
        let (lines, replies) = mpsc::channel();
        let output = Replies {
            history: Arc::clone(&history),
            lines,
            pending: Vec::new(),
        };
        let reports = Arc::clone(&history);
        let (done, result) = mpsc::channel();
        thread::spawn(move || {
            let search = move |request, report: &mut dyn FnMut(SearchReport)| {
                search(request, &mut |event| {
                    recorded(&reports).push(Recorded::Report(event.clone()));
                    report(event);
                });
            };
            drop(done.send(uci::run_session(input, output, "test", search)));
        });
        Self {
            history,
            input: Some(commands),
            replies,
            result,
        }
    }

    fn command(&self, command: &'static str) -> io::Result<()> {
        recorded(&self.history).push(Recorded::Command(command));
        self.input
            .as_ref()
            .ok_or_else(|| io::Error::other("input closed"))?
            .send_line(command)
    }

    fn eof(&mut self) {
        recorded(&self.history).push(Recorded::EndOfInput);
        drop(self.input.take());
    }

    fn reply(&self) -> io::Result<String> {
        receive(&self.replies)
    }

    fn finish(self) -> io::Result<Vec<String>> {
        receive(&self.result)??;
        let records = recorded(&self.history);
        let history: Vec<_> = records.iter().map(Recorded::as_event).collect();
        check(&history).map_err(|error| io::Error::other(format!("{error}: {history:?}")))?;
        Ok(self.replies.into_iter().collect())
    }
}

fn receive<T>(receiver: &mpsc::Receiver<T>) -> io::Result<T> {
    receiver
        .recv_timeout(HANG_TIMEOUT)
        .map_err(io::Error::other)
}

#[derive(Clone, Copy, Debug)]
enum Report {
    Depth,
    BestMove,
}

struct Script {
    steps: mpsc::Sender<Report>,
    reported: mpsc::Receiver<()>,
}

impl Script {
    fn start() -> (Session, Self) {
        let (steps, instructions) = mpsc::channel();
        let (sent, reported) = mpsc::channel();
        let session = Session::start(move |request, report| {
            let best_move = request.position.legal_moves().into_iter().next();
            let mut depth = 0;
            while let Ok(Report::Depth) = instructions.recv() {
                depth += 1;
                report(SearchReport::FinishedDepth(FinishedDepth {
                    depth,
                    nodes: 1,
                    elapsed: Duration::from_millis(1),
                    terminal_score: None,
                    variations: vec![(Score::Centipawns(0), best_move.into_iter().collect())],
                }));
                let _sent = sent.send(());
            }
            report(SearchReport::BestMove(best_move));
            let _sent = sent.send(());
        });
        (session, Self { steps, reported })
    }

    fn report(&self, report: Report) -> io::Result<()> {
        self.steps.send(report).map_err(io::Error::other)?;
        receive(&self.reported)
    }
}

#[test]
fn multipv_reaches_the_search_and_a_busy_setting_is_rejected() -> io::Result<()> {
    // D13 gives the output order. D46 keeps the setting for this go and the next.
    // Hold the worker before it reports so setoption runs during the search.
    let (started, requests) = mpsc::channel();
    let (release, proceed) = mpsc::channel();
    let session = Session::start(move |request, report| {
        if started.send(request.requested_variations.get()).is_err()
            || proceed.recv_timeout(HANG_TIMEOUT).is_err()
        {
            return;
        }
        report_scripted_variations(request.position, report);
    });

    session.command("setoption name MultiPV value 3")?;
    session.command("go depth 1")?;
    assert_eq!(receive(&requests)?, 3);
    session.command("setoption name MultiPV value 5")?;
    assert_eq!(
        session.reply()?,
        "info string setoption rejected: waiting for stop\n"
    );
    release.send(()).map_err(io::Error::other)?;
    assert_scripted_result(&session)?;

    session.command("setoption name MultiPV value 0")?;
    assert_eq!(
        session.reply()?,
        "info string setoption rejected: MultiPV requires an integer from 1 to 255\n"
    );
    session.command("go depth 1")?;
    assert_eq!(receive(&requests)?, 3);
    release.send(()).map_err(io::Error::other)?;
    assert_scripted_result(&session)?;
    session.command("quit")?;
    assert_eq!(session.finish()?, Vec::<String>::new());
    Ok(())
}

#[test]
fn multipv_setting_is_rejected_while_the_move_is_held() -> io::Result<()> {
    let (finished, requests) = mpsc::channel();
    let session = Session::start(move |request, report| {
        // The scripted search finishes at its depth cap, including BestMove.
        report_scripted_variations(request.position, report);
        assert!(finished.send(request.requested_variations.get()).is_ok());
    });

    session.command("setoption name MultiPV value 3")?;
    session.command("go infinite")?;
    assert_eq!(receive(&requests)?, 3);
    assert_scripted_variations(&session)?;
    // The BestMove report was queued before this command. A readyok here
    // confirms the session has consumed it and is holding the move.
    session.command("isready")?;
    assert_eq!(session.reply()?, "readyok\n");

    session.command("setoption name MultiPV value 5")?;
    session.command("isready")?;
    assert_eq!(
        session.reply()?,
        "info string setoption rejected: waiting for stop\n"
    );
    assert_eq!(session.reply()?, "readyok\n");
    session.command("stop")?;
    assert_eq!(session.reply()?, "bestmove e2e4\n");

    session.command("go depth 1")?;
    assert_eq!(receive(&requests)?, 3);
    assert_scripted_result(&session)?;
    session.command("quit")?;
    assert_eq!(session.finish()?, Vec::<String>::new());
    Ok(())
}

fn report_scripted_variations(position: rules::Position, report: &mut dyn FnMut(SearchReport)) {
    let variations: Vec<_> = [(17, "e2e4"), (8, "d2d4"), (-5, "g1f3")]
        .into_iter()
        .map(|(score, movement)| {
            (
                Score::Centipawns(score),
                scripted_variation(position, &[movement]),
            )
        })
        .collect();
    let best_move = variations
        .first()
        .and_then(|(_, moves)| moves.first())
        .copied();
    report(SearchReport::FinishedDepth(FinishedDepth {
        depth: 1,
        nodes: 30,
        elapsed: Duration::from_millis(2),
        terminal_score: None,
        variations,
    }));
    report(SearchReport::BestMove(best_move));
}

fn assert_scripted_result(session: &Session) -> io::Result<()> {
    assert_scripted_variations(session)?;
    assert_eq!(session.reply()?, "bestmove e2e4\n");
    Ok(())
}

fn assert_scripted_variations(session: &Session) -> io::Result<()> {
    for expected in [
        "info depth 1 multipv 1 score cp 17 nodes 30 nps 15000 time 2 pv e2e4\n",
        "info depth 1 multipv 2 score cp 8 nodes 30 nps 15000 time 2 pv d2d4\n",
        "info depth 1 multipv 3 score cp -5 nodes 30 nps 15000 time 2 pv g1f3\n",
    ] {
        assert_eq!(session.reply()?, expected);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug)]
enum Step {
    Command(&'static str),
    Eof,
    Report(Report),
}

const ALPHABET: [Step; 10] = [
    Step::Command(A),
    Step::Command(B),
    Step::Command("go"),
    Step::Command("go infinite"),
    Step::Command("stop"),
    Step::Command("isready"),
    Step::Command("quit"),
    Step::Eof,
    Step::Report(Report::Depth),
    Step::Report(Report::BestMove),
];

// Track which scripted reports can run within four steps. A go queued behind
// a stop cannot receive a scripted report until the first search reports its best move.
enum ScriptSearch {
    Idle,
    Finite,
    Infinite,
    Held,
}

impl ScriptSearch {
    fn command(&mut self, command: &str) {
        *self = match (&*self, command) {
            (Self::Idle, "go") | (Self::Infinite, "stop") => Self::Finite,
            (Self::Idle, "go infinite") => Self::Infinite,
            (Self::Held, "stop") => Self::Idle,
            _ => return,
        };
    }

    fn report(&mut self, script: &Script, report: Report) -> io::Result<()> {
        if matches!(self, Self::Finite | Self::Infinite) {
            script.report(report)?;
        }
        if matches!(report, Report::BestMove) {
            self.report_best_move();
        }
        Ok(())
    }

    fn report_best_move(&mut self) {
        *self = match self {
            Self::Finite => Self::Idle,
            Self::Infinite => Self::Held,
            Self::Idle | Self::Held => return,
        };
    }
}

fn play_short_history(steps: impl Iterator<Item = Step>) -> io::Result<()> {
    let (mut session, script) = Script::start();
    let mut search = ScriptSearch::Idle;
    let mut exiting = false;
    for step in steps.chain(std::iter::once(Step::Eof)) {
        match step {
            Step::Command(command) if !exiting => {
                session.command(command)?;
                exiting = command == "quit";
                search.command(command);
            }
            Step::Eof if !exiting => {
                session.eof();
                exiting = true;
            }
            Step::Report(Report::Depth) => {
                for _ in 0..3 {
                    search.report(&script, Report::Depth)?;
                }
            }
            Step::Report(Report::BestMove) => search.report(&script, Report::BestMove)?,
            Step::Command(_) | Step::Eof => {}
        }
    }
    drop(script);
    session.finish().map(drop)
}

#[test]
fn checks_every_sequence_up_to_four_steps() -> io::Result<()> {
    let started = Instant::now();
    let mut histories = 0;
    for length in 0..=4 {
        for history in 0..ALPHABET.len().pow(length) {
            let mut index = history;
            let steps = (0..length).map(|_| {
                let step = ALPHABET.get(index % ALPHABET.len()).copied();
                index /= ALPHABET.len();
                step
            });
            play_short_history(steps.flatten()).map_err(|error| {
                io::Error::other(format!("history {length}/{history}: {error}"))
            })?;
            histories += 1;
        }
    }
    assert_eq!(histories, 11_111);
    writeln!(
        io::stderr().lock(),
        "{histories} histories in {:?}",
        started.elapsed()
    )
}

#[test]
fn rejected_finite_replacements_leave_the_position_for_the_next_search() -> io::Result<()> {
    let session = Session::start(|request, report| {
        if request.limits.depth == Some(1) {
            wait_for_interrupt(&request.interrupt);
        }
        deepening::search(request, report);
    });
    for command in [A, "go depth 1", B, "go nodes 1"] {
        session.command(command)?;
    }
    assert_eq!(
        session.reply()?,
        "info string position rejected: waiting for stop\n"
    );
    assert_eq!(
        session.reply()?,
        "info string go rejected: waiting for stop\n"
    );
    session.command("stop")?;
    session.command("go depth 0")?;
    assert_eq!(session.reply()?, "bestmove h8g8\n");
    assert_eq!(session.reply()?, "bestmove h8g8\n");
    session.command("quit")?;
    assert_eq!(session.finish()?, Vec::<String>::new());
    Ok(())
}

#[test]
fn stop_releases_its_move_before_a_queued_position_and_go() -> io::Result<()> {
    let (session, script) = Script::start();
    for command in [A, "go infinite", "stop", B, "go", "quit"] {
        session.command(command)?;
    }
    script.report(Report::BestMove)?;
    script.report(Report::BestMove)?;
    assert_eq!(session.finish()?, ["bestmove h8g8\n", "bestmove h8h7\n"]);
    Ok(())
}

#[test]
fn acknowledged_commands_reach_the_channel_before_the_next_report() -> io::Result<()> {
    let (session, script) = Script::start();
    session.command(A)?;
    session.command("go")?;
    session.command("isready")?;
    // The first report can only run after the reader has acknowledged isready.
    script.report(Report::BestMove)?;
    assert_eq!(session.reply()?, "readyok\n");
    assert_eq!(session.reply()?, "bestmove h8g8\n");
    session.command("quit")?;
    assert_eq!(session.finish()?, Vec::<String>::new());
    Ok(())
}

fn wait_for_interrupt(interrupt: &std::sync::atomic::AtomicBool) {
    // The interrupt flag is the signal. The hang timeout catches a missing interrupt.
    let deadline = Instant::now() + HANG_TIMEOUT;
    while !interrupt.load(Ordering::Relaxed) {
        assert!(
            Instant::now() < deadline,
            "the session did not interrupt the search"
        );
        thread::yield_now();
    }
}

#[test]
fn answers_ready_while_real_search_is_held_then_interrupts_on_stop() -> io::Result<()> {
    let (started, hold) = mpsc::channel();
    let (resume, released) = mpsc::channel();
    let session = Session::start(move |request, report| {
        assert!(started.send(()).is_ok());
        assert!(receive(&released).is_ok());
        wait_for_interrupt(&request.interrupt);
        deepening::search(request, report);
    });
    session.command(PUZZLE)?;
    session.command("go infinite")?;
    receive(&hold)?;
    session.command("isready")?;
    assert_eq!(session.reply()?, "readyok\n");
    session.command("stop")?;
    resume.send(()).map_err(io::Error::other)?;
    assert_eq!(session.reply()?, "bestmove c5b4\n");
    session.command("stop")?;
    session.command("isready")?;
    assert_eq!(session.reply()?, "readyok\n");
    session.command("quit")?;
    assert_eq!(session.finish()?, Vec::<String>::new());
    Ok(())
}

#[test]
fn isready_leaves_a_finite_real_search_running() -> io::Result<()> {
    let (started, hold) = mpsc::channel();
    let (resume, released) = mpsc::channel();
    let session = Session::start(move |request, report| {
        assert!(started.send(()).is_ok());
        assert!(receive(&released).is_ok());
        assert!(!request.interrupt.load(Ordering::Relaxed));
        deepening::search(request, report);
    });
    session.command(A)?;
    session.command("go")?;
    receive(&hold)?;
    session.command("isready")?;
    assert_eq!(session.reply()?, "readyok\n");
    resume.send(()).map_err(io::Error::other)?;
    expect_depths(&session, 4)?;
    assert_eq!(session.reply()?, "bestmove h8g8\n");
    session.command("quit")?;
    assert_eq!(session.finish()?, Vec::<String>::new());
    Ok(())
}

fn expect_depths(session: &Session, last: u8) -> io::Result<()> {
    for depth in 1..=last {
        assert!(
            session
                .reply()?
                .starts_with(&format!("info depth {depth} "))
        );
    }
    Ok(())
}

#[test]
fn the_same_go_twice_reports_the_same_best_move_and_nodes() -> io::Result<()> {
    let session = Session::start(deepening::search);
    // The position after e2e4 is in the fisher-4yn score comparison corpus.
    session.command("position startpos moves e2e4")?;
    let go = || {
        session.command("go depth 4")?;
        let mut nodes = Vec::new();
        for depth in 1..=4 {
            let info = session.reply()?;
            assert!(info.starts_with(&format!("info depth {depth} ")), "{info}");
            let (_, nodes_and_rest) = info
                .split_once(" nodes ")
                .ok_or_else(|| io::Error::other(format!("no nodes in {info}")))?;
            let (count, _) = nodes_and_rest
                .split_once(' ')
                .ok_or_else(|| io::Error::other(format!("no node count in {info}")))?;
            nodes.push(count.to_owned());
        }
        let best_move = session.reply()?;
        assert!(best_move.starts_with("bestmove "), "{best_move}");
        Ok::<_, io::Error>((best_move, nodes))
    };
    assert_eq!(go()?, go()?);
    session.command("quit")?;
    assert_eq!(session.finish()?, Vec::<String>::new());
    Ok(())
}

fn first_starting_move() -> io::Result<String> {
    let best_move = rules::Position::starting()
        .legal_moves()
        .into_iter()
        .next()
        .ok_or_else(|| io::Error::other("no legal starting move"))?;
    Ok(format!("bestmove {}{}\n", best_move.from(), best_move.to()))
}

fn interrupt_real_search_on_exit(quit: bool) -> io::Result<()> {
    let mut session = Session::start(|request, report| {
        let interrupt = Arc::clone(&request.interrupt);
        deepening::search(request, &mut |event| {
            let at_boundary = matches!(
                event,
                SearchReport::FinishedDepth(FinishedDepth { depth: 1, .. })
            );
            report(event);
            if at_boundary {
                wait_for_interrupt(&interrupt);
            }
        });
    });
    session.command(PUZZLE)?;
    session.command("go infinite")?;
    let info = session.reply()?;
    assert!(
        info.starts_with("info depth 1 multipv 1 score cp -600 nodes 30 nps "),
        "{info}"
    );
    assert!(info.ends_with(" pv g5g2\n"), "{info}");
    if quit {
        session.command("quit")?;
    } else {
        session.eof();
    }
    assert_eq!(session.finish()?, ["bestmove g5g2\n"]);
    Ok(())
}

#[test]
fn quit_interrupts_real_search_and_exits_after_bestmove() -> io::Result<()> {
    interrupt_real_search_on_exit(true)
}

#[test]
fn eof_interrupts_real_search_and_exits_after_bestmove() -> io::Result<()> {
    interrupt_real_search_on_exit(false)
}

#[test]
fn stop_crossing_the_end_of_real_search_releases_once() -> io::Result<()> {
    let session = Session::start(|request, report| {
        let interrupt = Arc::clone(&request.interrupt);
        deepening::search(request, &mut |event| {
            if matches!(event, SearchReport::BestMove(_)) {
                wait_for_interrupt(&interrupt);
            }
            report(event);
        });
    });
    session.command(A)?;
    session.command("go")?;
    expect_depths(&session, 4)?;
    session.command("stop")?;
    session.command("quit")?;
    assert_eq!(session.finish()?, ["bestmove h8g8\n"]);
    Ok(())
}

#[test]
#[expect(clippy::panic, reason = "simulate a dying search thread")]
fn a_dying_search_returns_an_error_with_input_still_open() -> io::Result<()> {
    let session = Session::start(|_, _| panic!("search died"));
    session.command("go")?;
    let error = receive(&session.result)?
        .err()
        .ok_or_else(|| io::Error::other("session accepted a dying search"))?;
    assert_eq!(error.to_string(), "search thread died");
    Ok(())
}

#[test]
fn a_search_returning_without_a_best_move_is_an_error_with_input_still_open() -> io::Result<()> {
    let session = Session::start(|_, _| {});
    session.command("go")?;
    let error = receive(&session.result)?
        .err()
        .ok_or_else(|| io::Error::other("session accepted a search without a best move"))?;
    assert_eq!(error.to_string(), "search returned without a best move");
    Ok(())
}

#[test]
fn a_search_thread_dying_during_shutdown_returns_an_error() -> io::Result<()> {
    struct DiesOnDrop;

    impl Drop for DiesOnDrop {
        #[expect(
            clippy::panic,
            reason = "simulate a search thread dying as the session joins it"
        )]
        fn drop(&mut self) {
            panic!("search thread died on shutdown");
        }
    }

    let search = DiesOnDrop;
    let session = Session::start(move |request, report| {
        let _alive = &search;
        report(SearchReport::BestMove(
            request.position.legal_moves().into_iter().next(),
        ));
    });
    session.command("go")?;
    assert_eq!(session.reply()?, first_starting_move()?);
    session.command("quit")?;
    let error = receive(&session.result)?
        .err()
        .ok_or_else(|| io::Error::other("session accepted a dying search"))?;
    assert_eq!(error.to_string(), "search thread died");
    Ok(())
}

#[test]
fn reports_after_best_move_are_errors_with_input_still_open() -> io::Result<()> {
    for extra_report in [
        SearchReport::BestMove(None),
        SearchReport::FinishedDepth(FinishedDepth {
            depth: 1,
            nodes: 1,
            elapsed: Duration::from_millis(1),
            terminal_score: None,
            variations: vec![(Score::Centipawns(0), Vec::new())],
        }),
    ] {
        let session = Session::start(move |_, report| {
            report(SearchReport::BestMove(None));
            report(extra_report.clone());
        });
        session.command("go")?;
        let error = receive(&session.result)?
            .err()
            .ok_or_else(|| io::Error::other("session accepted a report after the best move"))?;
        assert_eq!(error.to_string(), "search reported after its best move");
        assert_eq!(session.reply()?, "bestmove 0000\n");
        assert!(session.replies.try_recv().is_err());
    }
    Ok(())
}

#[test]
fn holds_the_move_at_the_depth_cap_until_a_release_command() -> io::Result<()> {
    for command in [Some("stop"), Some("quit"), None] {
        let (held, at_depth_cap) = mpsc::channel();
        let mut session = Session::start(move |request, report| {
            // Drive the session with the scripted result of the private two-ply
            // depth cap. The deepening crate checks the same puzzle and counts.
            for (depth, cp, nodes, moves) in [
                (1, -600, 30, &["g5g2"][..]),
                (2, -700, 1308, &["c5f2", "g2g3"][..]),
            ] {
                let principal_variation = scripted_variation(request.position, moves);
                let best_move = principal_variation.first().copied();
                report(SearchReport::FinishedDepth(FinishedDepth {
                    depth,
                    nodes,
                    elapsed: Duration::from_millis(1),
                    terminal_score: None,
                    variations: vec![(Score::Centipawns(cp), principal_variation)],
                }));
                if depth == 2 {
                    report(SearchReport::BestMove(best_move));
                    assert!(held.send(()).is_ok());
                }
            }
        });
        session.command(PUZZLE)?;
        session.command("go infinite")?;
        expect_depths(&session, 2)?;
        receive(&at_depth_cap)?;
        session.command("isready")?;
        assert_eq!(session.reply()?, "readyok\n");
        match command {
            Some(command) => session.command(command)?,
            None => session.eof(),
        }
        assert_eq!(session.reply()?, "bestmove c5f2\n");
        if command == Some("stop") {
            session.command("stop")?;
            session.command("isready")?;
            assert_eq!(session.reply()?, "readyok\n");
            session.command("quit")?;
        }
        assert_eq!(session.finish()?, Vec::<String>::new());
    }
    Ok(())
}

#[test]
fn bare_go_matches_go_depth_four_with_the_same_elapsed_time() -> io::Result<()> {
    fn replies(command: &'static str) -> io::Result<Vec<String>> {
        let session = Session::start(|request, report| {
            deepening::search(request, &mut |mut event| {
                if let SearchReport::FinishedDepth(depth) = &mut event {
                    depth.elapsed = Duration::from_millis(u64::from(depth.depth));
                }
                report(event);
            });
        });
        session.command(A)?;
        session.command(command)?;
        let replies = (0..5).map(|_| session.reply()).collect::<io::Result<_>>()?;
        session.command("quit")?;
        assert_eq!(session.finish()?, Vec::<String>::new());
        Ok(replies)
    }
    assert_eq!(replies("go")?, replies("go depth 4")?);
    Ok(())
}

fn scripted_variation(mut position: rules::Position, moves: &[&str]) -> Vec<rules::Move> {
    moves
        .iter()
        .map(|text| {
            let movement = position
                .legal_moves()
                .into_iter()
                .find(|movement| format!("{}{}", movement.from(), movement.to()) == *text);
            let Some(movement) = movement else {
                unreachable!("the controlled schedule uses legal moves")
            };
            position = movement.play();
            movement
        })
        .collect()
}

#[test]
fn starting_position_reports_time_and_nps_at_the_depth_cap() -> io::Result<()> {
    let (held, at_depth_cap) = mpsc::channel();
    let session = Session::start(move |request, report| {
        // Schedule 4 supplies the totals. Deepening exercises its private
        // two-ply depth cap and controlled clock in its own tests.
        let principal_variation = scripted_variation(request.position, &["e2e4", "e7e5"]);
        report(SearchReport::FinishedDepth(FinishedDepth {
            depth: 1,
            nodes: 20,
            elapsed: Duration::from_millis(2),
            terminal_score: None,
            variations: vec![(
                Score::Centipawns(0),
                scripted_variation(request.position, &["e2e4"]),
            )],
        }));
        let best_move = principal_variation.first().copied();
        report(SearchReport::FinishedDepth(FinishedDepth {
            depth: 2,
            nodes: 440,
            elapsed: Duration::from_millis(5),
            terminal_score: None,
            variations: vec![(Score::Centipawns(0), principal_variation)],
        }));
        report(SearchReport::BestMove(best_move));
        assert!(held.send(()).is_ok());
    });
    session.command("position startpos")?;
    session.command("go infinite")?;
    assert_eq!(
        session.reply()?,
        "info depth 1 multipv 1 score cp 0 nodes 20 nps 10000 time 2 pv e2e4\n"
    );
    assert_eq!(
        session.reply()?,
        "info depth 2 multipv 1 score cp 0 nodes 440 nps 88000 time 5 pv e2e4 e7e5\n"
    );
    receive(&at_depth_cap)?;
    session.command("isready")?;
    assert_eq!(session.reply()?, "readyok\n");
    session.command("stop")?;
    assert_eq!(session.reply()?, "bestmove e2e4\n");
    session.command("quit")?;
    assert_eq!(session.finish()?, Vec::<String>::new());
    Ok(())
}

#[test]
fn holds_the_move_at_the_production_depth_cap() -> io::Result<()> {
    let (held, at_depth_cap) = mpsc::channel();
    let session = Session::start(move |request, report| {
        deepening::search(request, &mut |event| {
            let best_move = matches!(event, SearchReport::BestMove(_));
            report(event);
            if best_move {
                assert!(held.send(()).is_ok());
            }
        });
    });
    session.command("position fen 6bk/5p1p/5P1P/8/8/p1p5/P7/K7 b - - 0 1")?;
    session.command("go infinite")?;
    for depth in 1..=255 {
        let line = session.reply()?;
        let cp = if depth == 1 { 400 } else { 0 };
        assert!(
            line.starts_with(&format!(
                "info depth {depth} multipv 1 score cp {cp} nodes {depth} nps "
            )),
            "{line}"
        );
        assert!(line.ends_with(" pv c3c2\n"), "{line}");
    }
    receive(&at_depth_cap)?;
    session.command("isready")?;
    assert_eq!(session.reply()?, "readyok\n");
    session.command("stop")?;
    assert_eq!(session.reply()?, "bestmove c3c2\n");
    session.command("quit")?;
    assert_eq!(session.finish()?, Vec::<String>::new());
    Ok(())
}

#[path = "threaded_session/movetime.rs"]
mod movetime;

#[test]
fn commands_on_either_side_of_the_depth_limit() -> io::Result<()> {
    for command in [Some("stop"), Some("quit"), None] {
        depth_limit_with_command(command, true)?;
        depth_limit_with_command(command, false)?;
    }
    Ok(())
}

fn depth_limit_with_command(command: Option<&'static str>, command_first: bool) -> io::Result<()> {
    let mut session = search_to_depth_limit(command_first);
    session.command(PUZZLE)?;
    session.command("go depth 2")?;
    expect_puzzle_depth(&session, 1, -600, 30..=30, "g5g2")?;
    if !command_first {
        // The lower bound of 90 comes from fact 5 in expectations/uci-cutoff/README.md.
        expect_puzzle_depth(&session, 2, -700, 90..=1308, "c5f2 g2g3")?;
        assert_eq!(session.reply()?, "bestmove c5f2\n");
    }

    send_release_command(&mut session, command)?;
    if command_first {
        assert_eq!(session.reply()?, "bestmove g5g2\n");
    }
    if command == Some("stop") {
        session.command("isready")?;
        assert_eq!(session.reply()?, "readyok\n");
        session.command("quit")?;
    }
    assert_eq!(session.finish()?, Vec::<String>::new());
    Ok(())
}

fn send_release_command(session: &mut Session, command: Option<&'static str>) -> io::Result<()> {
    match command {
        Some(command) => session.command(command),
        None => {
            session.eof();
            Ok(())
        }
    }
}

fn search_to_depth_limit(command_first: bool) -> Session {
    Session::start(move |request, report| {
        let interrupt = Arc::clone(&request.interrupt);
        deepening::search(request, &mut |event| {
            let first_depth = matches!(
                event,
                SearchReport::FinishedDepth(FinishedDepth { depth: 1, .. })
            );
            report(event);
            // Deepening's private schedule can interrupt within depth 2.
            // At the session boundary, hold the worker after depth 1.
            if command_first && first_depth {
                wait_for_interrupt(&interrupt);
            }
        });
    })
}

fn expect_puzzle_depth(
    session: &Session,
    depth: u8,
    cp: i32,
    nodes: std::ops::RangeInclusive<u64>,
    pv: &str,
) -> io::Result<()> {
    let line = session.reply()?;
    let prefix = format!("info depth {depth} multipv 1 score cp {cp} nodes ");
    let Some((total, _)) = line
        .strip_prefix(&prefix)
        .and_then(|rest| rest.split_once(" nps "))
    else {
        return Err(io::Error::other(format!(
            "unexpected depth report: {line:?}"
        )));
    };
    let total = total.parse::<u64>().map_err(io::Error::other)?;
    assert!(nodes.contains(&total), "{line}");
    assert!(line.ends_with(&format!(" pv {pv}\n")));
    Ok(())
}
