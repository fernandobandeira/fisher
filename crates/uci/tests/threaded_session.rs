#[path = "support/search_history.rs"]
mod search_history;

use std::{
    io::{self, Write},
    sync::{Arc, Mutex, MutexGuard, atomic::Ordering, mpsc},
    thread,
    time::{Duration, Instant},
};

use deepening::{FinishedDepth, Score, SearchReport, SearchRequest};
use search_history::{Event, ScriptedInput, check, scripted_input};

const A: &str = "position fen 7k/8/5K1Q/8/8/8/8/8 b - - 0 1";
const B: &str = "position fen 7k/8/5K2/6Q1/8/8/8/8 b - - 0 1";
const TIMEOUT: Duration = Duration::from_secs(2);

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
    receiver.recv_timeout(TIMEOUT).map_err(io::Error::other)
}

#[derive(Clone, Copy, Debug)]
enum Report {
    Depth,
    End,
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
            let chosen = request.position.legal_moves().into_iter().next();
            let mut depth = 0;
            while let Ok(Report::Depth) = instructions.recv() {
                depth += 1;
                report(SearchReport::FinishedDepth(FinishedDepth {
                    depth,
                    score: Score::Centipawns(0),
                    nodes: 1,
                    elapsed: Duration::from_millis(1),
                    principal_variation: chosen.into_iter().collect(),
                }));
                let _sent = sent.send(());
            }
            report(SearchReport::Ended(chosen));
            let _sent = sent.send(());
        });
        (session, Self { steps, reported })
    }

    fn report(&self, report: Report) -> io::Result<()> {
        self.steps.send(report).map_err(io::Error::other)?;
        receive(&self.reported)
    }
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
    Step::Report(Report::End),
];

fn play_short_history(steps: impl Iterator<Item = Step>) -> io::Result<()> {
    let (mut session, script) = Script::start();
    // With three steps, a report action cannot follow a second accepted go:
    // releasing the first search and sending the second go already take three.
    let mut running = false;
    let mut exiting = false;
    for step in steps.chain(std::iter::once(Step::Eof)) {
        match step {
            Step::Command(command) if !exiting => {
                session.command(command)?;
                exiting = command == "quit";
                running |= matches!(command, "go" | "go infinite");
            }
            Step::Eof if !exiting => {
                session.eof();
                exiting = true;
            }
            Step::Report(report) if running => {
                script.report(report)?;
                running = matches!(report, Report::Depth);
            }
            Step::Command(_) | Step::Eof | Step::Report(_) => {}
        }
    }
    drop(script);
    session.finish().map(drop)
}

#[test]
fn checks_every_sequence_up_to_three_steps() -> io::Result<()> {
    let started = Instant::now();
    let mut histories = 0;
    for length in 0..=3 {
        for mut index in 0..ALPHABET.len().pow(length) {
            let steps = (0..length).map(|_| {
                let step = ALPHABET.get(index % ALPHABET.len()).copied();
                index /= ALPHABET.len();
                step
            });
            play_short_history(steps.flatten())?;
            histories += 1;
        }
    }
    assert_eq!(histories, 1111);
    writeln!(
        io::stderr().lock(),
        "{histories} histories in {:?}",
        started.elapsed()
    )
}

#[test]
fn rejected_finite_replacements_leave_the_position_for_the_next_search() -> io::Result<()> {
    let (mut session, script) = Script::start();
    for command in [A, "go", B, "go infinite", "isready"] {
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
    assert_eq!(session.reply()?, "readyok\n");
    script.report(Report::Depth)?;
    script.report(Report::End)?;
    assert!(session.reply()?.starts_with("info depth 1 "));
    assert_eq!(session.reply()?, "bestmove h8g8\n");
    session.command("go")?;
    script.report(Report::End)?;
    session.eof();
    assert_eq!(session.finish()?, ["bestmove h8g8\n"]);
    Ok(())
}

#[test]
fn stop_releases_its_move_before_a_queued_position_and_go() -> io::Result<()> {
    let (session, script) = Script::start();
    for command in [A, "go infinite", "stop", B, "go", "quit"] {
        session.command(command)?;
    }
    script.report(Report::End)?;
    script.report(Report::End)?;
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
    script.report(Report::End)?;
    assert_eq!(session.reply()?, "readyok\n");
    assert_eq!(session.reply()?, "bestmove h8g8\n");
    session.command("quit")?;
    assert_eq!(session.finish()?, Vec::<String>::new());
    Ok(())
}

fn wait_for_stop(stop: &std::sync::atomic::AtomicBool) {
    let deadline = Instant::now() + TIMEOUT;
    while !stop.load(Ordering::Relaxed) {
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
        wait_for_stop(&request.stop);
        deepening::search(request, report);
    });
    session.command("go infinite")?;
    receive(&hold)?;
    session.command("isready")?;
    assert_eq!(session.reply()?, "readyok\n");
    session.command("stop")?;
    resume.send(()).map_err(io::Error::other)?;
    session.command("stop")?;
    session.command("quit")?;
    let expected = first_starting_move()?;
    assert_eq!(session.finish()?, [expected]);
    Ok(())
}

fn first_starting_move() -> io::Result<String> {
    let chosen = rules::Position::starting()
        .legal_moves()
        .into_iter()
        .next()
        .ok_or_else(|| io::Error::other("no legal starting move"))?;
    Ok(format!("bestmove {}{}\n", chosen.from(), chosen.to()))
}

fn interrupt_real_search_on_exit(quit: bool) -> io::Result<()> {
    let (started, hold) = mpsc::channel();
    let mut session = Session::start(move |request, report| {
        assert!(started.send(()).is_ok());
        wait_for_stop(&request.stop);
        deepening::search(request, report);
    });
    session.command("go")?;
    receive(&hold)?;
    if quit {
        session.command("quit")?;
    } else {
        session.eof();
    }
    assert_eq!(session.finish()?, [first_starting_move()?]);
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
        let stop = Arc::clone(&request.stop);
        deepening::search(request, &mut |event| {
            if matches!(event, SearchReport::Ended(_)) {
                wait_for_stop(&stop);
            }
            report(event);
        });
    });
    session.command(A)?;
    session.command("go")?;
    assert!(session.reply()?.starts_with("info depth 4 "));
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
fn a_search_returning_without_ended_is_an_error_with_input_still_open() -> io::Result<()> {
    let session = Session::start(|_, _| {});
    session.command("go")?;
    let error = receive(&session.result)?
        .err()
        .ok_or_else(|| io::Error::other("session accepted a search without Ended"))?;
    assert_eq!(error.to_string(), "search returned without Ended");
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
        report(SearchReport::Ended(
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
