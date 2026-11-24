use std::{
    io::{self, BufRead, BufReader, Read, Write},
    sync::{Arc, atomic::AtomicBool, mpsc},
    thread,
    time::{Duration, Instant},
};

use deepening::{FinishedDepth, Limits, Score, SearchReport, SearchRequest};
use rules::Position;

use crate::{
    Response, SearchScore, UciMove,
    command::{Command, GoParameters, Input, PositionCommand, PositionSource, parse_command},
};

/// Runs a UCI session until quit, end of input, or an I/O error.
///
/// The reader can block independently of the command handler. Search runs on
/// the command thread and receives an owned request for each accepted go.
/// The callback queues finished depths and the end report in order. The search
/// must send exactly one `SearchReport::Ended` for each request, after its last
/// finished depth.
///
/// # Errors
///
/// Returns an error if the reader thread cannot start, input cannot be read,
/// or output cannot be written or flushed.
pub fn run_session(
    input: impl Read + Send + 'static,
    mut output: impl Write,
    version: &str,
    search: impl FnMut(SearchRequest, &mut dyn FnMut(SearchReport)) + Send + 'static,
) -> io::Result<()> {
    let (sender, lines) = mpsc::channel();
    thread::Builder::new().spawn(move || {
        if let Err(error) = read_input(BufReader::new(input), &sender) {
            drop(sender.send(Err(error)));
        }
    })?;

    answer_commands(lines, &mut output, version, search, Instant::now)
}

fn answer_commands(
    lines: impl IntoIterator<Item = io::Result<String>>,
    output: &mut impl Write,
    version: &str,
    mut search: impl FnMut(SearchRequest, &mut dyn FnMut(SearchReport)),
    now: impl Fn() -> Instant,
) -> io::Result<()> {
    let mut position = Position::starting();
    let mut held_move = None;
    // TODO(fisher-2aj.5): isready, stop and quit wait here for a running search to finish.
    for line in lines {
        match parse_command(&line?) {
            Input::Handled(Command::Uci) => {
                send(output, &Response::IdName(&format!("fisher {version}")))?;
                send(output, &Response::IdAuthor("Fernando Bandeira"))?;
                send(output, &Response::UciOk)?;
            }
            Input::Handled(Command::IsReady) => send(output, &Response::ReadyOk)?,
            Input::Handled(Command::Position(_)) if held_move.is_some() => {
                send(
                    output,
                    &Response::InfoString("position rejected: waiting for stop"),
                )?;
            }
            Input::Handled(Command::Go(_)) if held_move.is_some() => {
                send(
                    output,
                    &Response::InfoString("go rejected: waiting for stop"),
                )?;
            }
            Input::Handled(Command::Position(command)) => {
                match command
                    .map_err(|cause| cause.to_string())
                    .and_then(setup_position)
                {
                    Ok(next) => position = next,
                    Err(cause) => {
                        let message = format!("position rejected: {cause}");
                        send(output, &Response::InfoString(&message))?;
                    }
                }
            }
            Input::Handled(Command::Go(parameters)) => {
                let started = now();
                held_move = answer_go(output, position, &parameters, started, &mut search)?;
            }
            Input::Handled(Command::Stop) => {
                if let Some(chosen) = held_move.take() {
                    send(output, &Response::BestMove(chosen))?;
                }
            }
            Input::Handled(Command::Quit) => break,
            Input::Handled(Command::UciNewGame) | Input::Ignored | Input::Unknown => {}
        }
    }
    Ok(())
}

fn answer_go(
    output: &mut impl Write,
    position: Position,
    parameters: &GoParameters,
    started: Instant,
    search: &mut impl FnMut(SearchRequest, &mut dyn FnMut(SearchReport)),
) -> io::Result<Option<UciMove>> {
    let mut held_move = None;
    // TODO(fisher-7vo): Obey searchmoves and mate.
    // TODO(fisher-2ha): Wait for ponderhit when pondering.
    let request = SearchRequest {
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
        stop: Arc::new(AtomicBool::new(false)),
        started,
    };
    // While search runs on this thread, drain its reports before
    // reading the next command.
    let mut reports = Vec::new();
    search(request, &mut |report| reports.push(report));
    for report in reports {
        match report {
            SearchReport::FinishedDepth(depth) => send_search_info(output, &depth)?,
            SearchReport::Ended(chosen) => {
                let chosen = chosen.map_or(UciMove::Null, UciMove::from);
                if parameters.infinite {
                    held_move = Some(chosen);
                } else {
                    send(output, &Response::BestMove(chosen))?;
                }
            }
        }
    }
    Ok(held_move)
}

/// Terminal positions report `info depth 0 score mate 0` for checkmate
/// or `info depth 0 score cp 0` for stalemate.
fn send_search_info(output: &mut impl Write, result: &FinishedDepth) -> io::Result<()> {
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

fn read_input(
    mut input: impl BufRead,
    sender: &mpsc::Sender<io::Result<String>>,
) -> io::Result<()> {
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

fn send_line(sender: &mpsc::Sender<io::Result<String>>, line: &[u8]) -> bool {
    sender
        .send(Ok(String::from_utf8_lossy(line).into_owned()))
        .is_ok()
}

fn send(output: &mut impl Write, response: &Response<'_>) -> io::Result<()> {
    writeln!(output, "{response}")?;
    output.flush()
}

#[cfg(test)]
mod tests {
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
        let accepted = Instant::now();
        answer_commands(
            ["go infinite", "stop"].map(|command| Ok(command.to_owned())),
            &mut output,
            "test",
            |request, report| {
                let chosen_move = request.position.legal_moves().into_iter().next();
                assert_eq!(request.started, accepted);
                report(SearchReport::FinishedDepth(FinishedDepth {
                    depth: 4,
                    score: Score::Centipawns(0),
                    nodes: 206_603,
                    elapsed: Duration::from_micros(micros),
                    principal_variation: chosen_move.into_iter().collect(),
                }));
                report(SearchReport::Ended(chosen_move));
            },
            || accepted,
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
    fn infinite_search_holds_the_move_until_stop() -> io::Result<()> {
        let (sender, lines) = mpsc::channel();
        sender.send(Ok("go infinite".to_owned())).unwrap();
        let mut output = Vec::new();

        answer_commands(
            lines,
            &mut output,
            "test",
            move |request, report| {
                let chosen = request
                    .position
                    .legal_move(
                        rules::Square::parse("e2").unwrap(),
                        rules::Square::parse("e4").unwrap(),
                        None,
                    )
                    .unwrap();
                report(SearchReport::Ended(Some(chosen)));
                for command in ["isready", "stop", "quit"] {
                    sender.send(Ok(command.to_owned())).unwrap();
                }
            },
            Instant::now,
        )?;

        // The readiness response witnesses that the session still held the move.
        assert_eq!(output, b"readyok\nbestmove e2e4\n");
        Ok(())
    }

    #[test]
    fn crlf_split_across_reads_sends_one_line() -> io::Result<()> {
        let input = BufReader::with_capacity(1, &b"uci\r\nisready"[..]);
        let (sender, lines) = mpsc::channel();

        read_input(input, &sender)?;
        drop(sender);

        assert_eq!(
            lines.into_iter().collect::<io::Result<Vec<_>>>()?,
            ["uci", "isready"]
        );
        Ok(())
    }
}
