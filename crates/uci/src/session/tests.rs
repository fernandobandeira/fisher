#![cfg(test)]

use std::io::BufReader;

use deepening::Score;

use super::*;

#[test]
fn position_fen_uses_stockfish_clock_defaults() -> Result<(), String> {
    // Stockfish 19's FENs after g5g7, recorded in
    // expectations/uci-short-fen/README.md.
    for (command, expected) in [
        (
            "position fen 7k/8/5K2/6Q1/8/8/8/8 w - - moves g5g7",
            "7k/6Q1/5K2/8/8/8/8/8 b - - 1 1",
        ),
        (
            "position fen 7k/8/5K2/6Q1/8/8/8/8 w - - 17 moves g5g7",
            "7k/6Q1/5K2/8/8/8/8/8 b - - 18 1",
        ),
    ] {
        let Input::Handled(Command::Position(Ok(command))) = parse_command(command) else {
            return Err(format!("could not parse {command}"));
        };
        assert_eq!(setup_position(command)?.to_fen(), expected);
    }
    Ok(())
}

#[test]
fn position_fen_rejects_three_and_seven_fields() {
    // The rules parser rejected both forms before short FEN support.
    for fen in [
        "7k/8/5K2/6Q1/8/8/8/8 w -",
        "7k/8/5K2/6Q1/8/8/8/8 w - - 0 1 extra",
    ] {
        let result = setup_position(PositionCommand {
            source: PositionSource::Fen(fen.to_owned()),
            moves: Vec::new(),
        });
        assert!(result.is_err(), "{fen}");
    }
}

#[test]
fn position_fen_keeps_other_castling_fields_strict() {
    // Only the complete fields KK and H on an orthodox position are aliases.
    for (placement_and_color, field, diagnostic) in [
        ("4k2r/8/8/8/8/8/8/4K3 b", "kk", "duplicates"),
        ("4k2r/8/8/8/8/8/8/4K3 b", "h", "K, Q, k, q"),
        ("4k3/8/8/8/8/8/8/R3K3 w", "A", "K, Q, k, q"),
        ("r3k3/8/8/8/8/8/8/4K3 b", "a", "K, Q, k, q"),
        ("4k3/8/8/8/8/8/8/4K3 w", "H", "K, Q, k, q"),
        ("4k3/8/8/8/8/8/8/4K3 w", "KK", "duplicates"),
        ("4k3/8/8/8/8/8/8/4K2R w", "HH", "K, Q, k, q"),
        ("4k3/8/8/8/8/8/8/4K2R w", "KKK", "duplicates"),
        ("4k3/8/8/8/8/8/8/4K2R w", "KH", "K, Q, k, q"),
        ("4k3/8/8/8/8/8/8/R3K2R w", "HQ", "K, Q, k, q"),
        ("4k3/8/8/8/8/8/8/R3K2R w", "KKQ", "duplicates"),
        ("4k3/8/8/8/8/8/8/R3K2R w", "QQ", "duplicates"),
        ("4k3/8/8/8/8/8/8/3K3R w", "H", "K, Q, k, q"),
        ("4k3/8/8/8/8/8/8/3K3R w", "KK", "duplicates"),
        ("4k3/8/8/8/8/8/8/4K1R1 w", "G", "K, Q, k, q"),
        ("4k3/8/8/8/8/8/8/4K1R1 w", "H", "K, Q, k, q"),
    ] {
        let fen = format!("{placement_and_color} {field} - 0 1");
        let result = parse_uci_fen(fen.clone());
        assert!(
            matches!(&result, Err(error) if error.contains(diagnostic)),
            "{fen}: expected {diagnostic:?}, got {result:?}"
        );
    }
}

#[test]
fn node_rate_uses_reported_milliseconds() {
    assert_eq!(nodes_per_second(10, 2), 5000);
    assert_eq!(nodes_per_second(10, 3), 3333);
    assert_eq!(nodes_per_second(u64::MAX, 1000), u64::MAX);
    assert_eq!(nodes_per_second(u64::MAX, 1), u64::MAX);
}

#[test]
fn search_info_uses_reported_elapsed_time() -> io::Result<()> {
    // fisher-r6y D41 rounds elapsed milliseconds down.
    assert_search_info_time(3750, 3, 68_867_666)
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
            nodes: 206_603,
            elapsed: Duration::from_micros(micros),
            terminal_score: None,
            variations: vec![(
                Score::Centipawns(0),
                Position::starting()
                    .legal_moves()
                    .into_iter()
                    .take(1)
                    .collect(),
            )],
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
                Event::EndOfInput(_) | Event::Report(_) | Event::Error(_) =>
                    Err(io::Error::other("expected a line")),
            })
            .collect::<io::Result<Vec<_>>>()?,
        ["uci", "isready"]
    );
    Ok(())
}

#[test]
fn reader_error_during_stop_releases_the_move() -> io::Result<()> {
    reader_error_during_release("stop")
}

#[test]
fn reader_error_during_quit_releases_the_move() -> io::Result<()> {
    reader_error_during_release("quit")
}

fn reader_error_during_release(command: &str) -> io::Result<()> {
    let (sender, events) = mpsc::channel();
    let (requests, work) = mpsc::channel();
    let mut session = Session {
        position: Position::starting(),
        requested_variations: NonZeroU8::MIN,
        active: Active::Idle,
        deferred: VecDeque::new(),
        events,
        requests,
    };
    for event in [
        Event::Line("go".to_owned()),
        Event::Line(command.to_owned()),
        Event::EndOfInput(Err(io::ErrorKind::BrokenPipe.into())),
        Event::Report(SearchReport::BestMove(None)),
    ] {
        sender.send(event).map_err(io::Error::other)?;
    }
    drop(sender);
    let mut output = Vec::new();
    assert_read_error(session.run(&mut output, "test"), &output);
    let request = work.try_recv().map_err(io::Error::other)?;
    assert!(request.interrupt.load(Ordering::Relaxed));
    Ok(())
}

#[test]
fn reader_error_releases_a_held_move() -> io::Result<()> {
    let (sender, events) = mpsc::channel();
    let (requests, _work) = mpsc::channel();
    let mut session = Session {
        position: Position::starting(),
        requested_variations: NonZeroU8::MIN,
        active: Active::Held(UciMove::Null),
        deferred: VecDeque::new(),
        events,
        requests,
    };
    let input = read_input(BufReader::new(BrokenRead), &sender);
    sender
        .send(Event::EndOfInput(input))
        .map_err(io::Error::other)?;
    drop(sender);
    let mut output = Vec::new();
    assert_read_error(session.run(&mut output, "test"), &output);
    Ok(())
}

fn assert_read_error(result: io::Result<()>, output: &[u8]) {
    assert_eq!(
        result.err().map(|error| error.kind()),
        Some(io::ErrorKind::BrokenPipe)
    );
    assert_eq!(output, b"bestmove 0000\n");
}

#[test]
fn reports_failures_when_channels_disconnect() {
    let (sender, events) = mpsc::channel();
    let (requests, work) = mpsc::channel();
    let mut session = Session {
        position: Position::starting(),
        requested_variations: NonZeroU8::MIN,
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
        requested_variations: NonZeroU8::MIN,
        active: Active::Idle,
        deferred: VecDeque::new(),
        events,
        requests,
    };
    sender
        .send(Event::Line("go".to_owned()))
        .map_err(io::Error::other)?;
    sender
        .send(Event::Report(SearchReport::BestMove(None)))
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
