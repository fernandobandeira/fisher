use std::{io, sync::atomic::Ordering, time::Duration};

use deepening::{FinishedDepth, Limits, Score, SearchReport};
use rules::{Move, Position, Square};

#[test]
fn session_reports_the_scripted_search() -> io::Result<()> {
    let mut output = Vec::new();
    uci::run_session(
        &b"position startpos moves e2e4\ngo depth 2 nodes 440 movetime 50 wtime 1000 btime 2000 winc 30 binc 40 movestogo 5 infinite\nstop\n\
           position fen 7k/8/5K1Q/8/8/8/8/8 b - - 0 1\ngo nodes 0\nquit\n"[..],
        &mut output,
        "test",
        |request, report| {
            assert!(!request.stop.load(Ordering::Relaxed));
            if request.limits.nodes == Some(0) {
                assert_eq!(
                    request.limits,
                    Limits {
                        nodes: Some(0),
                        ..Limits::default()
                    }
                );
                // D20. This root has exactly one legal move, as in go-infinite.uci.
                report(SearchReport::Ended(request.position.legal_moves().into_iter().next()));
                return;
            }

            assert_eq!(
                request.limits,
                Limits {
                    depth: Some(2),
                    nodes: Some(440),
                    movetime: Some(Duration::from_millis(50)),
                    infinite: true,
                    wtime: Some(Duration::from_millis(1000)),
                    btime: Some(Duration::from_millis(2000)),
                    winc: Some(Duration::from_millis(30)),
                    binc: Some(Duration::from_millis(40)),
                    movestogo: Some(5),
                }
            );
            let chosen_move = legal_move(&request.position, "e7", "e5").unwrap();
            let reply = legal_move(&chosen_move.play(), "g1", "f3").unwrap();
            report(SearchReport::FinishedDepth(FinishedDepth {
                depth: 1,
                score: Score::Centipawns(17),
                nodes: 23,
                elapsed: Duration::from_millis(2),
                principal_variation: vec![chosen_move],
            }));
            report(SearchReport::FinishedDepth(FinishedDepth {
                depth: 2,
                score: Score::Centipawns(-6),
                nodes: 440,
                elapsed: Duration::from_millis(4),
                principal_variation: vec![chosen_move, reply],
            }));
            report(SearchReport::Ended(Some(chosen_move)));
        },
    )?;

    assert_eq!(
        String::from_utf8(output).map_err(io::Error::other)?,
        concat!(
            "info depth 1 multipv 1 score cp 17 nodes 23 nps 11500 time 2 pv e7e5\n",
            "info depth 2 multipv 1 score cp -6 nodes 440 nps 110000 time 4 pv e7e5 g1f3\n",
            "bestmove e7e5\n",
            "bestmove h8g8\n",
        )
    );
    Ok(())
}

fn legal_move(position: &Position, from: &str, to: &str) -> Option<Move> {
    position.legal_move(Square::parse(from)?, Square::parse(to)?, None)
}

fn session_output(
    input: &'static [u8],
    search: impl FnMut(deepening::SearchRequest, &mut dyn FnMut(SearchReport)) + Send + 'static,
) -> io::Result<String> {
    let mut output = Vec::new();
    uci::run_session(input, &mut output, "test", search)?;
    String::from_utf8(output).map_err(io::Error::other)
}

#[test]
fn answers_the_handshake_and_quits_before_a_later_command() -> io::Result<()> {
    // handshake.uci specifies the responses. unknown-input.uci accepts CRLF
    // and a lone CR (the `>CR isready` record).
    assert_eq!(
        session_output(
            b"ucinewgame\nunknown\nuci\r\nisready\rquit\nisready\n",
            |_, _| {}
        )?,
        "id name fisher test\nid author Fernando Bandeira\nuciok\nreadyok\n"
    );
    Ok(())
}

#[test]
fn waits_for_stop_before_accepting_another_go_or_position() -> io::Result<()> {
    // Held move and rejected commands from go-infinite.uci.
    let mut searched = false;
    let output = session_output(
        b"position fen 7k/8/5K1Q/8/8/8/8/8 b - - 0 1\ngo infinite\ngo\n          position startpos\nisready\nstop\nstop\n",
        move |request, report| {
            assert!(!searched);
            searched = true;
            report(SearchReport::Ended(legal_move(&request.position, "h8", "g8")));
        },
    )?;
    let responses: Vec<_> = output.lines().collect();
    let [first, second, "readyok", "bestmove h8g8"] = responses.as_slice() else {
        return Err(io::Error::other(format!(
            "unexpected responses: {output:?}"
        )));
    };
    for response in [first, second] {
        assert!(
            response.starts_with("info string ") && response.contains("waiting for stop"),
            "{response}"
        );
    }
    Ok(())
}

#[test]
fn rejected_positions_leave_the_last_accepted_position_in_place() -> io::Result<()> {
    // Diagnostics and retained sole legal move from rejected-position.uci.
    let output = session_output(
        b"position fen 7k/8/5K1Q/8/8/8/8/8 b - - 0 1\n          position fen 7k/8/5K1Q/8/8/8/8/7 b - - 0 1\n          position startpos moves e2e4 e7e5 e4e6\n          position startpos moves e2e4 e7e9\n          position fen\nposition\nposition startpos e2e4\n          position startpos moves e2e4 0000\ngo\n",
        |request, report| {
            assert_eq!(request.position.to_fen(), "7k/8/5K1Q/8/8/8/8/8 b - - 0 1");
            report(SearchReport::Ended(legal_move(&request.position, "h8", "g8")));
        },
    )?;
    let mut responses = output.lines();
    for expected in [
        "eight files",
        "illegal UCI move e4e6",
        "malformed UCI move",
        "expected FEN",
        "expected startpos or fen",
        "expected moves",
        "null move",
    ] {
        let response = responses.next().unwrap();
        assert!(
            response.starts_with("info string ") && response.contains(expected),
            "{response}"
        );
    }
    assert_eq!(responses.next(), Some("bestmove h8g8"));
    assert_eq!(responses.next(), None);
    Ok(())
}

#[test]
fn terminal_search_reports_omit_search_statistics_and_a_move() -> io::Result<()> {
    // The owner-approved responses in terminal-positions.uci.
    assert_eq!(
        session_output(
            b"position fen 7k/6Q1/5K2/8/8/8/8/8 b - - 0 1\ngo\n              position fen 7k/5K2/6Q1/8/8/8/8/8 b - - 0 1\ngo\n",
            deepening::search,
        )?,
        concat!("info depth 0 score mate 0\nbestmove 0000\n", "info depth 0 score cp 0\nbestmove 0000\n")
    );
    Ok(())
}

#[test]
fn reports_the_read_error_from_the_gui() {
    struct BrokenInput;
    impl io::Read for BrokenInput {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::ErrorKind::BrokenPipe.into())
        }
    }
    let error = uci::run_session(BrokenInput, io::sink(), "test", |_, _| {}).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
}

#[test]
fn reports_a_flush_failure_on_each_protocol_response() -> io::Result<()> {
    struct FailOnFlush(Option<usize>);
    impl io::Write for FailOnFlush {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            // Taking the countdown clears it even when this flush fails.
            if let Some(remaining) = self.0.take() {
                self.0 = Some(remaining.checked_sub(1).ok_or(io::ErrorKind::BrokenPipe)?);
            }
            Ok(())
        }
    }

    // Handshake and readiness, a rejected position, the held-move guards,
    // and the searched and terminal responses are specified in expectations/uci.
    const COMMANDS: &[u8] = b"uci\nisready\nposition\n        position fen 7k/8/5K1Q/8/8/8/8/8 b - - 0 1\n        go infinite\nposition startpos\ngo\nstop\ngo depth\nquit\n";
    let successful = session_output(COMMANDS, deepening::search)?;
    // Two ID responses and uciok, readyok, the position diagnostic, one info,
    // two rejected commands, the released move, and info/bestmove for finite go.
    assert_eq!(successful.lines().count(), 11);

    for response in 0..11 {
        let error = uci::run_session(
            COMMANDS,
            FailOnFlush(Some(response)),
            "test",
            deepening::search,
        )
        .unwrap_err();
        assert_eq!(
            error.kind(),
            io::ErrorKind::BrokenPipe,
            "response {response}"
        );
    }
    Ok(())
}
