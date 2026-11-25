use std::{
    collections::VecDeque,
    io,
    path::Path,
    process::Command,
    time::{Duration, Instant},
};

use super::{HANG_BOUND, TranscriptIo, exit_timeout, read_timeout, run_transcript};

// Python is already required by scripts/check. Decode the JSON with its standard
// library and send counted, NUL-separated fields to the Rust test.
const DECODE_FIXTURES: &str = r#"
import json
import sys

def emit(*fields):
    for field in fields:
        text = str(field)
        if "\0" in text:
            raise ValueError("fixture field contains NUL")
        sys.stdout.buffer.write(text.encode() + b"\0")

with open(sys.argv[1], encoding="utf-8") as source:
    for case in json.load(source)["cases"]:
        verdict = case["expected"]
        emit(case["name"], verdict["verdict"], verdict["reason"])
        emit("".join(line + "\n" for line in case["transcript"]))
        emit(len(case["script"]))
        for batch in case["script"]:
            emit(batch["after_send"], len(batch["events"]))
            for milliseconds, kind, value in batch["events"]:
                if batch.get("clock") != "observation" and milliseconds != 0:
                    raise ValueError("timed event outside an observation batch")
                emit(milliseconds, kind, value)
"#;

#[test]
fn repeated_searched_info_fixtures() -> io::Result<()> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../expectations/uci-runner/repeated-searched-info.json");
    let decoded = Command::new("python3")
        .args(["-c", DECODE_FIXTURES])
        .arg(path)
        .output()?;
    if !decoded.status.success() {
        return Err(io::Error::other(String::from_utf8_lossy(&decoded.stderr)));
    }
    let decoded = String::from_utf8(decoded.stdout).map_err(io::Error::other)?;
    let mut fields = decoded.split_terminator('\0');
    let mut failures = Vec::new();
    assert!(!decoded.is_empty(), "no repeated searched info fixtures");
    while let Some(name) = fields.next() {
        let verdict = field(&mut fields)?;
        let reason = field(&mut fields)?;
        let transcript = field(&mut fields)?;
        let mut script = Script::new(&mut fields)?;
        let mut started = false;
        let result = run_transcript(name, transcript, || {
            started = true;
            Ok(&mut script)
        });
        let observed = match (&result, started) {
            (Ok(_), _) => "accept",
            (Err(_), false) if script.sends == 0 => "load-reject",
            (Err(_), _) => "run-reject",
        };
        if observed != verdict {
            failures.push(format!(
                "{name}: expected {verdict} ({reason}), got {observed}: {result:?}"
            ));
        }
        failures.extend(timeout_failure(name, &result, &script));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

fn timeout_failure(
    name: &str,
    result: &io::Result<Vec<String>>,
    script: &Script<'_>,
) -> Option<String> {
    let timed_out = result.as_ref().err().map(io::Error::kind) == Some(io::ErrorKind::TimedOut);
    let elapsed = script.now.duration_since(script.start);
    (matches!(name, "delimiter-at-5100ms" | "flood-without-delimiter")
        && !(timed_out && elapsed == HANG_BOUND))
        .then(|| {
            format!(
                "{name}: expected stdout timeout at {HANG_BOUND:?}, got {result:?} at {elapsed:?}"
            )
        })
}

fn field<'a>(fields: &mut impl Iterator<Item = &'a str>) -> io::Result<&'a str> {
    fields
        .next()
        .ok_or_else(|| io::Error::other("incomplete decoded fixture"))
}

fn number<'a, T: std::str::FromStr>(fields: &mut impl Iterator<Item = &'a str>) -> io::Result<T>
where
    T::Err: std::error::Error + Send + Sync + 'static,
{
    field(fields)?.parse().map_err(io::Error::other)
}

enum Event<'a> {
    Stdout(&'a str),
    Eof,
    Exit(i32),
}

struct Batch<'a> {
    after_send: usize,
    events: Vec<(Duration, Event<'a>)>,
}

struct Script<'a> {
    batches: VecDeque<Batch<'a>>,
    events: VecDeque<(Instant, Event<'a>)>,
    start: Instant,
    now: Instant,
    sends: usize,
    input_closed: bool,
    eof: bool,
    exit: Option<i32>,
}

impl<'a> Script<'a> {
    fn new(fields: &mut impl Iterator<Item = &'a str>) -> io::Result<Self> {
        let mut batches = VecDeque::new();
        for _ in 0..number::<usize>(fields)? {
            let after_send = number(fields)?;
            let mut events = Vec::new();
            for _ in 0..number::<usize>(fields)? {
                // The reviewed fixture uses 5000 ms as one hang bound.
                // Preserve each event's position relative to that bound.
                let delay = HANG_BOUND * number::<u32>(fields)? / 5000;
                let event = match field(fields)? {
                    "stdout" => Event::Stdout(field(fields)?),
                    "eof" if field(fields)? == "None" => Event::Eof,
                    "exit" => Event::Exit(number(fields)?),
                    kind => return Err(io::Error::other(format!("invalid event {kind:?}"))),
                };
                events.push((delay, event));
            }
            batches.push_back(Batch { after_send, events });
        }
        let start = Instant::now();
        Ok(Self {
            batches,
            events: VecDeque::new(),
            start,
            now: start,
            sends: 0,
            input_closed: false,
            eof: false,
            exit: None,
        })
    }

    fn next_event(&mut self, deadline: Instant) -> Option<Event<'a>> {
        if self.now < deadline && self.events.front().is_some_and(|(at, _)| *at < deadline) {
            let (at, event) = self.events.pop_front()?;
            self.now = self.now.max(at);
            Some(event)
        } else {
            self.now = self.now.max(deadline);
            None
        }
    }
}

impl TranscriptIo for &mut Script<'_> {
    fn send(&mut self, _: &[u8]) -> io::Result<()> {
        if self.input_closed {
            return Err(io::Error::other("fisher stdin is closed"));
        }
        self.sends += 1;
        if self
            .batches
            .front()
            .is_some_and(|batch| batch.after_send == self.sends)
            && let Some(batch) = self.batches.pop_front()
        {
            // Only reads and the exit wait advance time. A timed batch starts
            // at the observation immediately after its triggering send.
            self.events.extend(
                batch
                    .events
                    .into_iter()
                    .map(|(delay, event)| (self.now + delay, event)),
            );
            self.events.make_contiguous().sort_by_key(|(at, _)| *at);
        }
        Ok(())
    }

    fn close_input(&mut self) {
        self.input_closed = true;
    }

    fn now(&self) -> Instant {
        self.now
    }

    fn read_reply(&mut self, deadline: Instant) -> io::Result<Option<String>> {
        while !self.eof {
            match self.next_event(deadline).ok_or_else(read_timeout)? {
                Event::Stdout(line) => return Ok(Some(line.to_owned())),
                Event::Eof => self.eof = true,
                Event::Exit(status) => self.exit = Some(status),
            }
        }
        Ok(None)
    }

    fn wait_for_success(&mut self, deadline: Instant) -> io::Result<()> {
        while self.exit.is_none() {
            if let Event::Exit(status) = self.next_event(deadline).ok_or_else(exit_timeout)? {
                self.exit = Some(status);
            }
        }
        match self.exit {
            Some(0) => Ok(()),
            Some(status) => Err(io::Error::other(format!(
                "fisher exited with status {status}"
            ))),
            None => Err(exit_timeout()),
        }
    }
}
