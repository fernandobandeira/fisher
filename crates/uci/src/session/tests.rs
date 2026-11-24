#![cfg(test)]

use std::io::BufReader;

use super::*;

#[test]
fn node_rate_uses_reported_milliseconds() {
    assert_eq!(nodes_per_second(10, 2), 5000);
    assert_eq!(nodes_per_second(10, 3), 3333);
    assert_eq!(nodes_per_second(u64::MAX, 1000), u64::MAX);
    assert_eq!(nodes_per_second(u64::MAX, 1), u64::MAX);
}

#[test]
fn search_info_uses_reported_elapsed_time() -> io::Result<()> {
    assert_search_info_time(3250, 3, 68_867_666)
}

#[test]
fn search_info_reports_one_millisecond_for_submillisecond_search() -> io::Result<()> {
    assert_search_info_time(250, 1, 206_603_000)
}

fn assert_search_info_time(micros: u64, time: u64, nps: u64) -> io::Result<()> {
    let mut output = Vec::new();
    send_search_info(
        &mut output,
        &FinishedDepth {
            depth: 4,
            score: Score::Centipawns(0),
            nodes: 206_603,
            elapsed: Duration::from_micros(micros),
            principal_variation: Position::starting()
                .legal_moves()
                .into_iter()
                .take(1)
                .collect(),
        },
    )?;
    let text = String::from_utf8(output).map_err(io::Error::other)?;
    assert!(
        text.starts_with(&format!(
            "info depth 4 multipv 1 score cp 0 nodes 206603 nps {nps} time {time} pv "
        )),
        "{text}"
    );
    Ok(())
}

#[test]
fn reader_stops_when_the_session_drops_its_receiver() -> io::Result<()> {
    let (sender, events) = mpsc::channel();
    drop(events);
    // The second read would fail if the reader does not stop on a closed receiver.
    let input = BufReader::new((&b"isready\n"[..]).chain(BrokenRead));
    read_input(input, &sender)
}

struct BrokenRead;

impl Read for BrokenRead {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Err(io::ErrorKind::BrokenPipe.into())
    }
}

#[test]
fn crlf_split_across_reads_sends_one_line() -> io::Result<()> {
    let input = BufReader::with_capacity(1, &b"uci\r\nisready"[..]);
    let (sender, lines) = mpsc::channel();

    read_input(input, &sender)?;
    drop(sender);

    assert_eq!(
        lines
            .into_iter()
            .map(|event| match event {
                Event::Line(line) => Ok(line),
                Event::EndOfInput | Event::Report(_) | Event::Error(_) =>
                    Err(io::Error::other("expected a line")),
            })
            .collect::<io::Result<Vec<_>>>()?,
        ["uci", "isready"]
    );
    Ok(())
}

#[test]
fn reader_error_during_stop_ends_the_session() -> io::Result<()> {
    let (sender, events) = mpsc::channel();
    let (requests, work) = mpsc::channel();
    let mut session = Session {
        position: Position::starting(),
        active: Active::Idle,
        deferred: VecDeque::new(),
        events,
        requests,
    };
    sender
        .send(Event::Line("go".to_owned()))
        .map_err(io::Error::other)?;
    sender
        .send(Event::Line("stop".to_owned()))
        .map_err(io::Error::other)?;
    sender
        .send(Event::Error(io::ErrorKind::BrokenPipe.into()))
        .map_err(io::Error::other)?;
    let result = session.run(&mut io::sink(), "test");
    assert_eq!(
        result.err().map(|error| error.kind()),
        Some(io::ErrorKind::BrokenPipe)
    );
    let request = work.try_recv().map_err(io::Error::other)?;
    assert!(request.stop.load(Ordering::Relaxed));
    Ok(())
}

#[test]
fn reports_failures_when_channels_disconnect() {
    let (sender, events) = mpsc::channel();
    let (requests, work) = mpsc::channel();
    let mut session = Session {
        position: Position::starting(),
        active: Active::Idle,
        deferred: VecDeque::new(),
        events,
        requests,
    };
    drop(sender);
    drop(work);
    let mut output = io::sink();
    assert!(session.run(&mut output, "test").is_err());
    assert!(session.go(&mut output, &GoParameters::default()).is_err());
    assert!(session.release(&mut output).is_err());
}

#[test]
fn a_report_write_error_ends_the_command_loop() -> io::Result<()> {
    let (sender, events) = mpsc::channel();
    let (requests, _work) = mpsc::channel();
    let mut session = Session {
        position: Position::starting(),
        active: Active::Idle,
        deferred: VecDeque::new(),
        events,
        requests,
    };
    sender
        .send(Event::Line("go".to_owned()))
        .map_err(io::Error::other)?;
    sender
        .send(Event::Report(SearchReport::Ended(None)))
        .map_err(io::Error::other)?;
    drop(sender);
    struct ClosedOutput;
    impl Write for ClosedOutput {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::ErrorKind::BrokenPipe.into())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let result = session.run(&mut ClosedOutput, "test");
    assert_eq!(
        result.err().map(|error| error.kind()),
        Some(io::ErrorKind::BrokenPipe)
    );
    Ok(())
}
