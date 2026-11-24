//! Session rules 1–10 (fisher-8ie, D24 and D25), read 2026-11-24.
//! Source: April 2006 UCI, <https://backscattering.de/chess/uci/>.
//! Record commands and reports in their shared input order and record
//! replies at the writer, including each newline. Finish recording when
//! the session returns successfully. Synchronize the inputs and replies
//! when a history requires a particular order.
//! Commands use the ordinary UCI spelling. Parser edge cases belong to
//! transcripts. A report belongs to the one accepted search in progress.

use std::collections::VecDeque;

use deepening::{FinishedDepth, Score, SearchReport};
use shakmaty::{CastlingMode, Chess, Position as _, fen::Fen, uci::UciMove};

#[derive(Clone, Debug)]
pub(super) enum Event<'a> {
    Command(&'a str),
    EndOfInput,
    Report(SearchReport),
    Reply(&'a str),
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

struct Search {
    infinite: bool,
    stopped: bool,
    ended: Option<String>,
    info: VecDeque<String>,
    last_move: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CommandReply {
    Ready,
    PositionRejected,
    InvalidPosition,
    GoRejected,
    IdName,
    IdAuthor,
    UciOk,
}

#[derive(Default)]
struct Session<'a> {
    position: Chess,
    rejected_position: Option<Chess>,
    search: Option<Search>,
    command_replies: VecDeque<CommandReply>,
    deferred: VecDeque<Option<&'a str>>,
    idle_stop: bool,
    exiting: bool,
}

pub(super) fn check(history: &[Event<'_>]) -> Result<(), Violation> {
    let mut session = Session::default();
    for (index, event) in history.iter().enumerate() {
        session.observe(event).map_err(|(rule, detail)| Violation {
            rule,
            event: index + 1,
            detail,
        })?;
    }
    session.finish().map_err(|(rule, detail)| Violation {
        rule,
        event: history.len() + 1,
        detail,
    })
}

impl<'a> Session<'a> {
    fn observe(&mut self, event: &Event<'a>) -> Result<(), Fault> {
        match event {
            Event::Command(command) => self.receive(Some(command)),
            Event::EndOfInput => self.receive(None),
            Event::Report(report) => self.report(report)?,
            Event::Reply(line) => self.reply(line)?,
        }
        Ok(())
    }

    fn receive(&mut self, command: Option<&'a str>) {
        if self
            .search
            .as_ref()
            .is_some_and(|search| search.stopped || (!search.infinite && search.ended.is_some()))
        {
            self.deferred.push_back(command);
        } else if !self.exiting {
            self.command(command);
        }
    }

    fn command(&mut self, command: Option<&str>) {
        let mut words = command.unwrap_or("quit").split_ascii_whitespace();
        match words.next() {
            Some("go") if self.search.is_some() => {
                self.command_replies.push_back(CommandReply::GoRejected);
            }
            Some("go") => {
                self.idle_stop = false;
                self.search = Some(Search {
                    infinite: words.any(|word| word == "infinite"),
                    stopped: false,
                    ended: None,
                    info: VecDeque::new(),
                    last_move: None,
                });
            }
            Some("position") if self.search.is_some() => {
                self.rejected_position = position(words);
                self.command_replies
                    .push_back(CommandReply::PositionRejected);
            }
            Some("position") => {
                if let Some(next) = position(words) {
                    self.position = next;
                    self.rejected_position = None;
                } else {
                    self.command_replies
                        .push_back(CommandReply::PositionRejected);
                }
            }
            Some("stop") => {
                if let Some(search) = &mut self.search {
                    search.stopped = true;
                } else {
                    self.idle_stop = true;
                }
            }
            Some("quit") => self.exiting = true,
            Some("isready") => self.command_replies.push_back(CommandReply::Ready),
            Some("uci") => self.command_replies.extend([
                CommandReply::IdName,
                CommandReply::IdAuthor,
                CommandReply::UciOk,
            ]),
            _ => {}
        }
    }

    fn report(&mut self, report: &SearchReport) -> Result<(), Fault> {
        let Some(search) = &mut self.search else {
            return Err((8, "a search reported without an accepted go"));
        };
        if search.ended.is_some() {
            return Err((3, "a search reported after Ended"));
        }
        match report {
            SearchReport::FinishedDepth(depth) => {
                search.info.push_back(info(depth));
                search.last_move = depth.principal_variation.first().map(movement);
            }
            SearchReport::Ended(chosen) => {
                search.ended = Some(chosen.as_ref().map_or_else(|| "0000".to_owned(), movement));
            }
        }
        Ok(())
    }

    fn reply(&mut self, line: &str) -> Result<(), Fault> {
        if self.exiting && self.search.is_none() && self.command_replies.is_empty() {
            return Err((9, "output after the final reply on exit"));
        }
        let body = line
            .strip_suffix('\n')
            .and_then(|body| (!body.contains(['\n', '\r'])).then_some(body))
            .ok_or((10, "expected one newline-terminated reply"))?;
        match body.split_once(' ').unwrap_or((body, "")) {
            ("bestmove", text) if text.parse::<UciMove>().is_ok() => self.bestmove(text),
            ("info", text) if text.starts_with("depth ") => self.search_info(body),
            ("readyok", "") if body == "readyok" => self.command_reply(CommandReply::Ready),
            ("uciok", "") if body == "uciok" => self.command_reply(CommandReply::UciOk),
            ("id", text) if payload(text, "name ") => self.command_reply(CommandReply::IdName),
            ("id", text) if payload(text, "author ") => self.command_reply(CommandReply::IdAuthor),
            ("info", "string position rejected: waiting for stop") => {
                self.command_reply(CommandReply::PositionRejected)
            }
            ("info", "string go rejected: waiting for stop") => {
                self.command_reply(CommandReply::GoRejected)
            }
            ("info", text) if payload(text, "string position rejected: ") => {
                self.command_reply(CommandReply::InvalidPosition)
            }
            ("info", text) if payload(text, "string ") && self.idle_stop => {
                Err((4, "idle stop wrote a reply"))
            }
            ("info", text) if payload(text, "string ") => Err((10, "unexpected diagnostic")),
            _ => Err((10, "expected one whole session reply")),
        }
    }

    fn command_reply(&mut self, reply: CommandReply) -> Result<(), Fault> {
        if self.command_replies.front() == Some(&reply) {
            self.command_replies.pop_front();
            if reply == CommandReply::Ready {
                self.idle_stop = false;
            }
            return Ok(());
        }
        if reply == CommandReply::Ready && !self.command_replies.is_empty() {
            return Err((7, "readyok preceded an earlier command reply"));
        }
        if self.idle_stop && !self.command_replies.contains(&reply) {
            return Err((4, "idle stop wrote a reply"));
        }
        if matches!(
            reply,
            CommandReply::PositionRejected
                | CommandReply::InvalidPosition
                | CommandReply::GoRejected
        ) {
            return Err((8, "wrong or extra rejection of position or go"));
        }
        Err((
            if reply == CommandReply::Ready { 6 } else { 10 },
            "unsolicited command reply",
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
        if self.command_replies.contains(&CommandReply::Ready) {
            return Err((6, "bestmove preceded readyok for a search still running"));
        }
        if !self.command_replies.is_empty() {
            return Err((
                8,
                "bestmove preceded the rejection or reply to an earlier command",
            ));
        }
        if search.infinite && !search.stopped && !self.exiting {
            return Err((2, "go infinite released its move before stop or exit"));
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
        if search.ended.as_deref() != Some(text) {
            return Err((3, "bestmove did not match the search's Ended report"));
        }
        if (search.stopped || self.exiting)
            && search
                .last_move
                .as_deref()
                .is_some_and(|movement| movement != text)
        {
            return Err((3, "bestmove did not use the last finished depth"));
        }
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
        if self.command_replies.contains(&CommandReply::Ready)
            || self.deferred.contains(&Some("isready"))
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
        if !self.command_replies.is_empty() {
            return Err((8, "missing command reply"));
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

fn position(mut words: std::str::SplitAsciiWhitespace<'_>) -> Option<Chess> {
    let mut position = match words.next()? {
        "startpos" => Chess::default(),
        "fen" => words
            .by_ref()
            .take(6)
            .collect::<Vec<_>>()
            .join(" ")
            .parse::<Fen>()
            .ok()?
            .into_position(CastlingMode::Standard)
            .ok()?,
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

// D41 and expectations/README.md specify the fields and their spelling.
fn info(depth: &FinishedDepth) -> String {
    let score = match depth.score {
        Score::Centipawns(value) => format!("cp {value}"),
        Score::Mate { plies, winning } => {
            let moves = i32::from(plies.div_ceil(2));
            let moves = if winning { moves } else { -moves };
            format!("mate {moves}")
        }
    };
    if depth.depth == 0 {
        return format!("info depth 0 score {score}");
    }
    let time = depth.elapsed.as_millis().clamp(1, u128::from(u64::MAX));
    let nps = (u128::from(depth.nodes) * 1000 / time).min(u128::from(u64::MAX));
    let pv = depth
        .principal_variation
        .iter()
        .map(movement)
        .collect::<Vec<_>>()
        .join(" ");
    format!(
        "info depth {} multipv 1 score {score} nodes {} nps {nps} time {time} pv {pv}",
        depth.depth, depth.nodes
    )
}
