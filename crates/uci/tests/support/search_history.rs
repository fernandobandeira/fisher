//! Session rules 1–10 (fisher-8ie, D24 and D25), read 2026-11-24.
//! Source: April 2006 UCI, <https://backscattering.de/chess/uci/>.
//! Record commands in input order, reports in report order, and responses
//! at the writer, including each newline. Finish recording when the session
//! returns successfully. Before stepping a scripted finished depth or `BestMove`,
//! wait until every earlier recorded command has reached the ordered command
//! channel. Use `scripted_input` to wait for the reader to ask for the next
//! line, after it has enqueued the one just sent. That acknowledgement holds
//! because the session's reader sends each complete line before it reads
//! again; the scripted-input test cannot observe it while the search runs on
//! the command thread, so fisher-2aj.5 must show it failing on a reader that
//! acknowledges before sending. Record a `Report` event before passing that
//! report to the session's callback, so its `info` line or `bestmove` cannot
//! be recorded ahead of it.
//! Commands recorded before `BestMove` are handled before it, subject to the
//! stop barrier. After a finite search reports `BestMove`, commands may be
//! handled on either side of `bestmove` in input order (fisher-8ie rules 6–8).
//! This checker reads fisher-2aj.5's "until that search has reported" and
//! the `SearchReport` seam as requiring `BestMove` before `bestmove`. Infinite work
//! also needs stop or exit (rule 2 and D25).
//! Commands use ordinary UCI spelling. A report belongs to the one
//! accepted search in progress.

use std::{
    collections::VecDeque,
    io::{self, Cursor, Read},
    sync::mpsc,
    time::Duration,
};

use deepening::{FinishedDepth, Score, SearchReport};
use shakmaty::{CastlingMode, Chess, Position as _, fen::Fen, uci::UciMove};

pub(super) const HANG_TIMEOUT: Duration = Duration::from_secs(30);

pub(super) struct ScriptedInput {
    lines: mpsc::Sender<String>,
    next_read: mpsc::Receiver<()>,
}

impl ScriptedInput {
    /// Send one complete line and wait until the reader asks for another.
    pub(super) fn send_line(&self, line: &str) -> io::Result<()> {
        assert!(!line.contains(['\r', '\n']), "send exactly one command");
        self.lines
            .send(format!("{line}\n"))
            .map_err(io::Error::other)?;

        self.next_read
            .recv_timeout(HANG_TIMEOUT)
            .map_err(io::Error::other)
    }
}

/// The reader signals when it asks for more input after reading a line.
pub(super) fn scripted_input() -> (impl Read + Send + 'static, ScriptedInput) {
    let (lines, input) = mpsc::channel();
    let (ready, next_read) = mpsc::channel();
    (
        ChannelReader {
            input,
            ready,
            pending: Cursor::new(String::new()),
        },
        ScriptedInput { lines, next_read },
    )
}

struct ChannelReader {
    input: mpsc::Receiver<String>,
    ready: mpsc::Sender<()>,
    pending: Cursor<String>,
}

impl Read for ChannelReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if buffer.is_empty() {
            return Ok(0);
        }
        let read = self.pending.read(buffer)?;
        if read != 0 {
            return Ok(read);
        }
        if !self.pending.get_ref().is_empty() && self.ready.send(()).is_err() {
            return Ok(0);
        }
        let Ok(line) = self.input.recv() else {
            return Ok(0);
        };
        self.pending = Cursor::new(line);
        self.pending.read(buffer)
    }
}

#[derive(Clone, Debug)]
pub(super) enum Event<'a> {
    Command(&'a str),
    EndOfInput,
    Report(SearchReport),
    Response(&'a str),
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct Violation {
    pub(super) rule: u8,
    event: usize,
    detail: &'static str,
}

impl std::fmt::Display for Violation {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            output,
            "rule {} at event {}: {}",
            self.rule, self.event, self.detail
        )
    }
}

type Fault = (u8, &'static str);

const NOT_A_RESPONSE: Fault = (10, "expected one whole session response");

#[derive(Clone)]
struct Search {
    infinite: bool,
    stopped: bool,
    readiness_requested: bool,
    best_move: Option<String>,
    info: VecDeque<String>,
    last_move: Option<String>,
}

impl Search {
    fn report_depth(&mut self, depth: &FinishedDepth) -> Result<(), Fault> {
        if depth.depth == 0 {
            let Some(score) = depth.terminal_score.filter(|_| depth.variations.is_empty()) else {
                return Err((3, "invalid terminal depth report"));
            };
            self.info
                .push_back(format!("info depth 0 score {}", score_text(score)));
            self.last_move = None;
            return Ok(());
        }

        if depth.terminal_score.is_some() || depth.variations.is_empty() {
            return Err((3, "invalid searched depth report"));
        }
        for (index, (score, variation)) in depth.variations.iter().enumerate() {
            if variation.is_empty() {
                return Err((3, "invalid searched depth report"));
            }
            self.info
                .push_back(info(depth, index + 1, *score, variation));
        }
        self.last_move = depth
            .variations
            .first()
            .and_then(|(_, moves)| moves.first())
            .map(movement);
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CommandResponse {
    Ready,
    PositionRejected,
    InvalidPosition,
    GoRejected,
    SetOptionRejected,
    InvalidMultiPv,
    IdName,
    IdAuthor,
    MultiPvOption,
    UciOk,
}

#[derive(Clone, Default)]
struct Session<'a> {
    position: Chess,
    rejected_position: Option<Chess>,
    search: Option<Search>,
    command_responses: VecDeque<CommandResponse>,
    deferred: VecDeque<&'a str>,
    idle_stop: bool,
    exiting: bool,
    last_bestmove: Option<String>,
}

pub(super) fn check(history: &[Event<'_>]) -> Result<(), Violation> {
    // Keep each possible handling order until a response rules it out.
    // Once one command waits for bestmove, later commands wait behind it.
    let mut sessions = vec![Session::default()];
    for (index, event) in history.iter().enumerate() {
        let mut next = Vec::new();
        let mut failure = None;
        for mut session in sessions {
            if let Some(command) = input(event)
                && session.can_wait_for_bestmove()
            {
                let mut waiting = session.clone();
                waiting.deferred.push_back(command);
                next.push(waiting);
            }
            match session.observe(event) {
                Ok(()) => next.push(session),
                Err(fault) => {
                    failure = Some(fault);
                }
            }
        }
        if next.is_empty()
            && let Some((rule, detail)) = failure
        {
            return Err(Violation {
                rule,
                event: index + 1,
                detail,
            });
        }
        sessions = next;
    }
    let mut result = Ok(());
    for session in sessions {
        match session.finish() {
            Ok(()) => return Ok(()),
            Err((rule, detail)) => {
                result = Err(Violation {
                    rule,
                    event: history.len() + 1,
                    detail,
                });
            }
        }
    }
    result
}

fn input<'a>(event: &Event<'a>) -> Option<&'a str> {
    match event {
        Event::Command(command) => Some(command),
        Event::EndOfInput => Some("quit"),
        Event::Report(_) | Event::Response(_) => None,
    }
}

impl<'a> Session<'a> {
    fn observe(&mut self, event: &Event<'a>) -> Result<(), Fault> {
        match event {
            Event::Command(command) => self.receive(command),
            Event::EndOfInput => self.receive("quit"),
            Event::Report(report) => self.report(report)?,
            Event::Response(line) => self.response(line)?,
        }
        Ok(())
    }

    fn can_wait_for_bestmove(&self) -> bool {
        !self.exiting
            && self.deferred.is_empty()
            && self.search.as_ref().is_some_and(|search| {
                !search.stopped && !search.infinite && search.best_move.is_some()
            })
    }

    fn receive(&mut self, command: &'a str) {
        if self.exiting {
            return;
        }
        if !self.deferred.is_empty() || self.search.as_ref().is_some_and(|search| search.stopped) {
            self.deferred.push_back(command);
        } else {
            self.command(command);
        }
    }

    fn command(&mut self, command: &str) {
        let mut words = command.split_ascii_whitespace();
        match words.next() {
            Some("go") if self.search.is_some() => {
                self.command_responses
                    .push_back(CommandResponse::GoRejected);
            }
            Some("go") => {
                self.idle_stop = false;
                self.search = Some(Search {
                    infinite: words.any(|word| word == "infinite"),
                    stopped: false,
                    readiness_requested: false,
                    best_move: None,
                    info: VecDeque::new(),
                    last_move: None,
                });
            }
            Some("position") if self.search.is_some() => {
                self.rejected_position = position(words);
                self.command_responses
                    .push_back(CommandResponse::PositionRejected);
            }
            Some("position") => {
                if let Some(next) = position(words) {
                    self.position = next;
                    self.rejected_position = None;
                } else {
                    self.command_responses
                        .push_back(CommandResponse::InvalidPosition);
                }
            }
            Some("setoption") => {
                self.command_responses
                    .extend(multipv_response(words, self.search.is_some()));
            }
            Some("stop") => {
                if let Some(search) = &mut self.search {
                    search.stopped = true;
                } else {
                    self.idle_stop = true;
                }
            }
            Some("quit") => {
                self.exiting = true;
                self.deferred.clear();
            }
            Some("isready") => {
                if let Some(search) = &mut self.search {
                    search.readiness_requested = true;
                }
                self.command_responses.push_back(CommandResponse::Ready);
            }
            Some("uci") => self.command_responses.extend([
                CommandResponse::IdName,
                CommandResponse::IdAuthor,
                CommandResponse::MultiPvOption,
                CommandResponse::UciOk,
            ]),
            _ => {}
        }
    }

    fn report(&mut self, report: &SearchReport) -> Result<(), Fault> {
        let Some(search) = &mut self.search else {
            return Err((8, "a search reported without an accepted go"));
        };
        if search.best_move.is_some() {
            return Err((3, "a search reported after BestMove"));
        }
        match report {
            SearchReport::FinishedDepth(depth) => search.report_depth(depth)?,
            SearchReport::BestMove(best_move) => {
                search.best_move = Some(
                    best_move
                        .as_ref()
                        .map_or_else(|| "0000".to_owned(), movement),
                );
            }
        }
        Ok(())
    }

    fn response(&mut self, line: &str) -> Result<(), Fault> {
        if self.exiting && self.search.is_none() && self.command_responses.is_empty() {
            return Err((9, "output after the final response on exit"));
        }
        let body = line
            .strip_suffix('\n')
            .and_then(|body| (!body.contains(['\n', '\r'])).then_some(body))
            .ok_or((10, "expected one newline-terminated response"))?;
        match body.split_once(' ').unwrap_or((body, "")) {
            ("bestmove", text) if text.parse::<UciMove>().is_ok() => self.bestmove(text),
            ("info", text) if text.starts_with("depth ") => self.search_info(body),
            ("info", text) => self.info_string(text),
            _ => self.reply(body),
        }
    }

    /// Judges an `info` response other than search information: a
    /// rejection, or a diagnostic nothing asked for.
    fn info_string(&mut self, text: &str) -> Result<(), Fault> {
        let response = match text {
            "string position rejected: waiting for stop" => CommandResponse::PositionRejected,
            "string go rejected: waiting for stop" => CommandResponse::GoRejected,
            "string setoption rejected: waiting for stop" => CommandResponse::SetOptionRejected,
            "string setoption rejected: MultiPV requires an integer from 1 to 255" => {
                CommandResponse::InvalidMultiPv
            }
            _ if payload(text, "string position rejected: ") => CommandResponse::InvalidPosition,
            _ if payload(text, "string ") && self.idle_stop => {
                return Err((4, "idle stop wrote a response"));
            }
            _ if payload(text, "string ") => return Err((10, "unexpected diagnostic")),
            _ => return Err(NOT_A_RESPONSE),
        };
        self.command_response(response)
    }

    /// Judges a reply to `isready` or `uci`.
    fn reply(&mut self, body: &str) -> Result<(), Fault> {
        let response = match (body, body.strip_prefix("id ")) {
            ("readyok", _) => CommandResponse::Ready,
            ("option name MultiPV type spin default 1 min 1 max 255", _) => {
                CommandResponse::MultiPvOption
            }
            ("uciok", _) => CommandResponse::UciOk,
            (_, Some(text)) if payload(text, "name ") => CommandResponse::IdName,
            (_, Some(text)) if payload(text, "author ") => CommandResponse::IdAuthor,
            _ => return Err(NOT_A_RESPONSE),
        };
        self.command_response(response)
    }

    fn command_response(&mut self, response: CommandResponse) -> Result<(), Fault> {
        if self.command_responses.front() == Some(&response) {
            self.command_responses.pop_front();
            return Ok(());
        }
        if self.idle_stop && !self.command_responses.contains(&response) {
            return Err((4, "idle stop wrote a response"));
        }
        if response == CommandResponse::Ready && !self.command_responses.is_empty() {
            return Err((7, "readyok preceded an earlier command response"));
        }
        if matches!(
            response,
            CommandResponse::PositionRejected
                | CommandResponse::InvalidPosition
                | CommandResponse::GoRejected
        ) {
            return Err((8, "wrong or extra rejection of position or go"));
        }
        Err((
            if response == CommandResponse::Ready {
                6
            } else {
                10
            },
            "unsolicited command response",
        ))
    }

    fn search_info(&mut self, body: &str) -> Result<(), Fault> {
        let Some(search) = &mut self.search else {
            return Err(if self.idle_stop {
                (4, "idle stop wrote search information")
            } else {
                (3, "search information without an outstanding go")
            });
        };
        if search.info.pop_front().as_deref() != Some(body) {
            return Err((3, "info did not match the next finished depth"));
        }
        Ok(())
    }

    fn bestmove(&mut self, text: &str) -> Result<(), Fault> {
        let search = self
            .search
            .as_ref()
            .ok_or((1, "bestmove without an outstanding go"))?;
        // Once the next search reports `BestMove` with the same move, the
        // history cannot tell which search produced a matching bestmove.
        if search.best_move.is_none() && self.last_bestmove.as_deref() == Some(text) {
            return Err((
                1,
                "duplicate bestmove before the next search reported BestMove",
            ));
        }
        if search.infinite && !search.stopped && !self.exiting {
            return Err(if search.readiness_requested {
                (6, "isready ended an infinite search")
            } else {
                (2, "go infinite released its move before stop or exit")
            });
        }
        if let Some(response) = self.command_responses.front() {
            let rule = match response {
                CommandResponse::Ready => 6,
                CommandResponse::PositionRejected
                | CommandResponse::InvalidPosition
                | CommandResponse::GoRejected => 8,
                CommandResponse::SetOptionRejected
                | CommandResponse::InvalidMultiPv
                | CommandResponse::IdName
                | CommandResponse::IdAuthor
                | CommandResponse::MultiPvOption
                | CommandResponse::UciOk => 10,
            };
            return Err((rule, "bestmove preceded an outstanding command response"));
        }
        if !legal(&self.position, text) {
            if self
                .rejected_position
                .as_ref()
                .is_some_and(|position| legal(position, text))
            {
                return Err((8, "bestmove used the rejected position"));
            }
            return Err((5, "bestmove is illegal in the accepted position"));
        }
        if !search.info.is_empty() {
            return Err((3, "bestmove preceded info for a finished depth"));
        }
        if search.best_move.as_deref() != Some(text) {
            return Err((3, "bestmove did not match the search's BestMove report"));
        }
        if (search.stopped || self.exiting)
            && search
                .last_move
                .as_deref()
                .is_some_and(|movement| movement != text)
        {
            return Err((3, "bestmove did not use the last finished depth"));
        }
        self.last_bestmove = Some(text.to_owned());
        self.search = None;
        while !self.exiting && !self.search.as_ref().is_some_and(|search| search.stopped) {
            let Some(command) = self.deferred.pop_front() else {
                break;
            };
            self.command(command);
        }
        Ok(())
    }

    fn finish(&self) -> Result<(), Fault> {
        let mut deferred_before_exit = self
            .deferred
            .iter()
            .copied()
            .take_while(|command| command.split_ascii_whitespace().next() != Some("quit"));
        if self.command_responses.contains(&CommandResponse::Ready)
            || deferred_before_exit
                .any(|text| text.split_ascii_whitespace().next() == Some("isready"))
        {
            return Err((6, "missing readyok"));
        }
        if let Some(search) = &self.search {
            if self.exiting {
                return Err((9, "quit or end of input did not release bestmove"));
            }
            if search.stopped && !self.deferred.is_empty() {
                return Err((8, "stop did not release bestmove before later commands"));
            }
            if search.stopped {
                return Err((3, "stop did not release bestmove"));
            }
            return Err((1, "accepted go has no bestmove"));
        }
        if !self.command_responses.is_empty() {
            return Err((8, "missing command response"));
        }
        if !self.exiting {
            return Err((9, "history has no quit or end of input"));
        }
        Ok(())
    }
}

fn payload(text: &str, prefix: &str) -> bool {
    text.strip_prefix(prefix)
        .is_some_and(|value| !value.trim().is_empty())
}

// The name is case insensitive and ends at the value marker. Only a declared
// name can cause a response, and the busy rejection takes precedence.
fn multipv_response(
    mut words: std::str::SplitAsciiWhitespace<'_>,
    busy: bool,
) -> Option<CommandResponse> {
    if words.next() != Some("name") || !words.next()?.eq_ignore_ascii_case("MultiPV") {
        return None;
    }
    let valid = match words.next() {
        Some("value") => {
            words
                .next()
                .is_some_and(|value| value.parse::<std::num::NonZeroU8>().is_ok())
                && words.next().is_none()
        }
        None => false,
        Some(_) => return None,
    };
    if busy {
        Some(CommandResponse::SetOptionRejected)
    } else {
        (!valid).then_some(CommandResponse::InvalidMultiPv)
    }
}

fn position(words: std::str::SplitAsciiWhitespace<'_>) -> Option<Chess> {
    let mut words = words.peekable();
    let mut position = match words.next()? {
        "startpos" => Chess::default(),
        "fen" => {
            let mut fen = Vec::new();
            while words.peek().is_some_and(|word| *word != "moves") {
                fen.push(words.next()?);
            }
            if !(4..=6).contains(&fen.len()) {
                return None;
            }
            if fen.get(4).is_some_and(|clock| fen_decimal(clock).is_none())
                || fen.get(5).is_some_and(|number| {
                    fen_decimal(number)
                        .and_then(std::num::NonZeroU64::new)
                        .is_none()
                })
            {
                return None;
            }
            // The checker uses Chess for move legality. The engine's u64 clocks
            // can exceed the range of Chess's counters, so pass only the first four fields.
            fen.truncate(4);
            fen.join(" ")
                .parse::<Fen>()
                .ok()?
                .into_position(CastlingMode::Standard)
                .ok()?
        }
        _ => return None,
    };
    match words.next() {
        Some("moves") => {
            for text in words {
                let movement = text.parse::<UciMove>().ok()?.to_move(&position).ok()?;
                position.play_unchecked(movement);
            }
        }
        None => {}
        Some(_) => return None,
    }
    Some(position)
}

fn fen_decimal(text: &str) -> Option<u64> {
    if !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

fn legal(position: &Chess, text: &str) -> bool {
    if text == "0000" {
        position.legal_moves().is_empty()
    } else {
        text.parse::<UciMove>()
            .is_ok_and(|movement| movement.to_move(position).is_ok())
    }
}

fn movement(movement: &rules::Move) -> String {
    let promotion = movement
        .promotion()
        .map_or("", |promotion| match promotion {
            rules::Promotion::Queen => "q",
            rules::Promotion::Rook => "r",
            rules::Promotion::Bishop => "b",
            rules::Promotion::Knight => "n",
        });
    format!("{}{}{promotion}", movement.from(), movement.to())
}

fn score_text(score: Score) -> String {
    match score {
        Score::Centipawns(value) => format!("cp {value}"),
        Score::Mate { plies, winning } => {
            let moves = i32::from(plies.div_ceil(2));
            let moves = if winning { moves } else { -moves };
            format!("mate {moves}")
        }
    }
}

// D41 and expectations/README.md specify the fields and their spelling.
fn info(depth: &FinishedDepth, multipv: usize, score: Score, variation: &[rules::Move]) -> String {
    let score = score_text(score);
    let time = depth.elapsed.as_millis().clamp(1, u128::from(u64::MAX));
    let nps = (u128::from(depth.nodes) * 1000 / time).min(u128::from(u64::MAX));
    let pv = variation.iter().map(movement).collect::<Vec<_>>().join(" ");
    format!(
        "info depth {} multipv {multipv} score {score} nodes {} nps {nps} time {time} pv {pv}",
        depth.depth, depth.nodes
    )
}
