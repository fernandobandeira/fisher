use std::{
    collections::VecDeque,
    io::{self, BufRead, BufReader, Read, Write},
    panic::{self, AssertUnwindSafe},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};

use deepening::{FinishedDepth, Limits, Score, SearchReport, SearchRequest};
use rules::Position;

use crate::{
    Response, SearchScore, UciMove,
    command::{
        Command, GoParameters, Input, PositionCommand, PositionSource, PositionSyntaxError,
        parse_command,
    },
};

/// Runs a UCI session until quit, end of input, or an I/O error.
///
/// The reader and search thread send events into one ordered stream. Only the
/// command handler writes output. Each search must send one `SearchReport::BestMove`
/// after its last finished depth and return.
///
/// # Errors
///
/// Returns an error if a thread cannot start, the search dies or returns
/// without a best move, the search reports after its best move, input cannot
/// be read, or output cannot be written or flushed.
pub fn run_session(
    input: impl Read + Send + 'static,
    mut output: impl Write,
    version: &str,
    search: impl FnMut(SearchRequest, &mut dyn FnMut(SearchReport)) + Send + 'static,
) -> io::Result<()> {
    let (sender, events) = mpsc::channel();
    let reader = sender.clone();
    thread::Builder::new().spawn(move || {
        let event = Event::EndOfInput(read_input(BufReader::new(input), &reader));
        drop(reader.send(event));
    })?;

    let (requests, work) = mpsc::channel();
    let worker = thread::Builder::new().spawn(move || {
        run_searches(work, &sender, search).inspect_err(|error| {
            drop(sender.send(Event::Error(io::Error::other(error.to_string()))));
        })
    })?;
    let mut session = Session {
        position: Position::starting(),
        active: Active::Idle,
        deferred: VecDeque::new(),
        events,
        requests,
    };
    let result = session.run(&mut output, version);
    session.active.interrupt();
    drop(session);
    let joined = worker
        .join()
        .map_err(|_| io::Error::other("search thread died"));
    result.and(joined.flatten())
}

enum Event {
    Line(String),
    EndOfInput(io::Result<()>),
    Report(SearchReport),
    Error(io::Error),
}

fn run_searches(
    work: mpsc::Receiver<SearchRequest>,
    events: &mpsc::Sender<Event>,
    mut search: impl FnMut(SearchRequest, &mut dyn FnMut(SearchReport)),
) -> io::Result<()> {
    for request in work {
        let mut progress = SearchProgress::Running;
        panic::catch_unwind(AssertUnwindSafe(|| {
            search(request, &mut |report| progress.send(report, events));
        }))
        .map_err(|_| io::Error::other("search thread died"))?;
        progress.finish()?;
    }
    Ok(())
}

enum SearchProgress {
    Running,
    BestMoveReported,
    ReportedAfterBestMove,
}

impl SearchProgress {
    fn send(&mut self, report: SearchReport, events: &mpsc::Sender<Event>) {
        if !matches!(self, Self::Running) {
            *self = Self::ReportedAfterBestMove;
            return;
        }
        if matches!(report, SearchReport::BestMove(_)) {
            *self = Self::BestMoveReported;
        }
        drop(events.send(Event::Report(report)));
    }

    fn finish(self) -> io::Result<()> {
        match self {
            Self::Running => Err(io::Error::other("search returned without a best move")),
            Self::BestMoveReported => Ok(()),
            Self::ReportedAfterBestMove => {
                Err(io::Error::other("search reported after its best move"))
            }
        }
    }
}

enum Active {
    Idle,
    Searching {
        interrupt: Arc<AtomicBool>,
        infinite: bool,
    },
    Held(UciMove),
}

impl Active {
    fn interrupt(&self) {
        if let Self::Searching { interrupt, .. } = self {
            interrupt.store(true, Ordering::Relaxed);
        }
    }
}

struct Session {
    position: Position,
    active: Active,
    deferred: VecDeque<Event>,
    events: mpsc::Receiver<Event>,
    requests: mpsc::Sender<SearchRequest>,
}

impl Session {
    fn run(&mut self, output: &mut dyn Write, version: &str) -> io::Result<()> {
        loop {
            match self.next_event()? {
                Event::Line(line) => match parse_command(&line) {
                    Input::Handled(Command::Quit) => return self.quit(output),
                    Input::Handled(command) => self.command(output, version, command)?,
                    Input::Ignored | Input::Unknown => {}
                },
                Event::EndOfInput(result) => return self.release(output).and(result),
                Event::Report(report) => self.report(output, report)?,
                Event::Error(error) => return Err(error),
            }
        }
    }

    fn receive(&self) -> io::Result<Event> {
        self.events.recv().map_err(io::Error::other)
    }

    fn next_event(&mut self) -> io::Result<Event> {
        self.deferred.pop_front().map_or_else(|| self.receive(), Ok)
    }

    fn command(
        &mut self,
        output: &mut dyn Write,
        version: &str,
        command: Command,
    ) -> io::Result<()> {
        match command {
            Command::Uci => {
                send(output, &Response::IdName(&format!("fisher {version}")))?;
                send(output, &Response::IdAuthor("Fernando Bandeira"))?;
                send(output, &Response::UciOk)
            }
            Command::IsReady => send(output, &Response::ReadyOk),
            Command::Position(command) => self.position(output, command),
            Command::Go(parameters) => self.go(output, &parameters),
            Command::Stop | Command::Quit => self.release(output),
            Command::UciNewGame => Ok(()),
        }
    }

    fn position(
        &mut self,
        output: &mut dyn Write,
        command: Result<PositionCommand, PositionSyntaxError>,
    ) -> io::Result<()> {
        if !matches!(self.active, Active::Idle) {
            return send(
                output,
                &Response::InfoString("position rejected: waiting for stop"),
            );
        }
        match command
            .map_err(|cause| cause.to_string())
            .and_then(setup_position)
        {
            Ok(next) => {
                self.position = next;
                Ok(())
            }
            Err(cause) => send(
                output,
                &Response::InfoString(&format!("position rejected: {cause}")),
            ),
        }
    }

    fn go(&mut self, output: &mut dyn Write, parameters: &GoParameters) -> io::Result<()> {
        if !matches!(self.active, Active::Idle) {
            return send(
                output,
                &Response::InfoString("go rejected: waiting for stop"),
            );
        }
        let request = search_request(self.position, parameters);
        self.active = Active::Searching {
            interrupt: Arc::clone(&request.interrupt),
            infinite: parameters.infinite,
        };
        self.requests.send(request).map_err(io::Error::other)
    }

    fn report(&mut self, output: &mut dyn Write, report: SearchReport) -> io::Result<()> {
        match report {
            SearchReport::FinishedDepth(depth) => send_search_info(output, &depth),
            SearchReport::BestMove(best_move) => {
                let best_move = best_move.map_or(UciMove::Null, UciMove::from);
                if matches!(self.active, Active::Searching { infinite: true, .. }) {
                    self.active = Active::Held(best_move);
                    Ok(())
                } else {
                    self.active = Active::Idle;
                    // Throwaway only: prove the real-clock match reports a late move.
                    if self.position.to_fen().split_whitespace().nth(1) == Some("w")
                        && self.position.to_fen().split_whitespace().nth(5) == Some("10")
                    {
                        std::thread::sleep(std::time::Duration::from_secs(2));
                    }
                    send(output, &Response::BestMove(best_move))
                }
            }
        }
    }

    fn quit(&mut self, output: &mut dyn Write) -> io::Result<()> {
        self.release(output)?;
        for event in self.deferred.drain(..) {
            if let Event::EndOfInput(result) = event {
                return result;
            }
        }
        Ok(())
    }

    fn release(&mut self, output: &mut dyn Write) -> io::Result<()> {
        self.active.interrupt();
        while matches!(self.active, Active::Searching { .. }) {
            match self.receive()? {
                Event::Report(report) => self.report(output, report)?,
                Event::Error(error) => return Err(error),
                event @ (Event::Line(_) | Event::EndOfInput(_)) => self.deferred.push_back(event),
            }
        }
        if let Active::Held(best_move) = std::mem::replace(&mut self.active, Active::Idle) {
            send(output, &Response::BestMove(best_move))?;
        }
        Ok(())
    }
}

fn search_request(position: Position, parameters: &GoParameters) -> SearchRequest {
    // TODO(fisher-7vo): Obey searchmoves and mate.
    // TODO(fisher-2ha): Wait for ponderhit when pondering.
    SearchRequest {
        position,
        limits: Limits {
            depth: parameters.depth,
            nodes: parameters.nodes,
            movetime: parameters.movetime.map(Duration::from_millis),
            infinite: parameters.infinite,
            wtime: parameters.wtime.map(Duration::from_millis),
            btime: parameters.btime.map(Duration::from_millis),
            winc: parameters.winc.map(Duration::from_millis),
            binc: parameters.binc.map(Duration::from_millis),
            movestogo: parameters.movestogo,
        },
        interrupt: Arc::new(AtomicBool::new(false)),
        started: Instant::now(),
    }
}

/// Terminal positions report `info depth 0 score mate 0` for checkmate
/// or `info depth 0 score cp 0` for stalemate.
fn send_search_info(output: &mut dyn Write, result: &FinishedDepth) -> io::Result<()> {
    let time_ms = u64::try_from(result.elapsed.as_millis().max(1)).unwrap_or(u64::MAX);
    let pv: Vec<_> = result
        .principal_variation
        .iter()
        .copied()
        .map(UciMove::from)
        .collect();
    let nodes = (result.depth > 0).then_some(result.nodes);
    let score = match result.score {
        Score::Centipawns(value) => SearchScore::Centipawns(value),
        Score::Mate { plies, winning } => SearchScore::Mate { plies, winning },
    };
    send(
        output,
        &Response::SearchInfo {
            depth: result.depth,
            score,
            nodes,
            time_ms: nodes.map(|_| time_ms),
            nps: nodes.map(|nodes| nodes_per_second(nodes, time_ms)),
            pv: if result.depth == 0 { &[] } else { &pv },
        },
    )
}

fn nodes_per_second(nodes: u64, time_ms: u64) -> u64 {
    let rate = u128::from(nodes) * 1000 / u128::from(time_ms);
    u64::try_from(rate).unwrap_or(u64::MAX)
}

fn setup_position(command: PositionCommand) -> Result<Position, String> {
    let mut position = match command.source {
        PositionSource::Startpos => Position::starting(),
        PositionSource::Fen(fen) => {
            Position::from_fen(&fen).map_err(|cause| format!("invalid FEN: {cause}"))?
        }
    };
    for uci_move in command.moves {
        let UciMove::Normal {
            from,
            to,
            promotion,
        } = uci_move
        else {
            return Err("null move in position move list".to_owned());
        };
        position = position
            .legal_move(from, to, promotion)
            .ok_or_else(|| format!("illegal UCI move {uci_move}"))?
            .play();
    }
    Ok(position)
}

fn read_input(mut input: impl BufRead, sender: &mpsc::Sender<Event>) -> io::Result<()> {
    let mut line = Vec::new();
    let mut previous_cr = false;
    loop {
        let bytes = input.fill_buf()?;
        if bytes.is_empty() {
            break;
        }
        for &byte in bytes {
            // A CR already completes the line, even across buffer boundaries.
            if byte == b'\r' || (byte == b'\n' && !previous_cr) {
                if !send_line(sender, &line) {
                    return Ok(());
                }
                line.clear();
            } else if byte != b'\n' {
                line.push(byte);
            }
            previous_cr = byte == b'\r';
        }
        let consumed = bytes.len();
        input.consume(consumed);
    }
    if !line.is_empty() {
        let _sent = send_line(sender, &line);
    }
    Ok(())
}

fn send_line(sender: &mpsc::Sender<Event>, line: &[u8]) -> bool {
    sender
        .send(Event::Line(String::from_utf8_lossy(line).into_owned()))
        .is_ok()
}

fn send(output: &mut dyn Write, response: &Response<'_>) -> io::Result<()> {
    writeln!(output, "{response}")?;
    output.flush()
}

#[cfg(test)]
mod tests;
