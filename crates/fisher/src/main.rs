use std::{
    io::{self, BufRead, Write},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use rules::Position;
use search::{Score, SearchResult, minimax};
use uci::{
    Command, Input, PositionCommand, PositionSource, Response, SearchScore, UciMove, parse_command,
};

fn main() -> io::Result<()> {
    let (sender, lines) = mpsc::channel();
    thread::Builder::new().spawn(move || {
        if let Err(error) = read_input(io::stdin().lock(), &sender) {
            drop(sender.send(Err(error)));
        }
    })?;

    let clock_start = Instant::now();
    answer_commands(lines, &mut io::stdout().lock(), || clock_start.elapsed())
}

fn answer_commands(
    lines: impl IntoIterator<Item = io::Result<String>>,
    output: &mut impl Write,
    now: impl Fn() -> Duration,
) -> io::Result<()> {
    let mut position = Position::starting();
    let mut held_move = None;
    // TODO(fisher-2aj.5): isready, stop and quit wait here for a running search to finish.
    for line in lines {
        match parse_command(&line?) {
            Input::Handled(Command::Uci) => {
                send(
                    output,
                    &Response::IdName(concat!("fisher ", env!("CARGO_PKG_VERSION"))),
                )?;
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
                // TODO(fisher-7vo): Obey the supplied search limits.
                // TODO(fisher-2ha): Wait for ponderhit when pondering.
                const DEPTH: u8 = 4;
                let started = now();
                let result = minimax(&position, DEPTH);
                let elapsed = now().saturating_sub(started);
                let time_ms = u64::try_from(elapsed.as_millis().max(1)).unwrap_or(u64::MAX);
                let pv: Vec<_> = result
                    .principal_variation
                    .iter()
                    .copied()
                    .map(UciMove::from)
                    .collect();
                send(output, &search_info(DEPTH, &result, time_ms, &pv))?;
                let chosen = result.chosen_move.map_or(UciMove::Null, UciMove::from);
                if parameters.infinite {
                    held_move = Some(chosen);
                } else {
                    send(output, &Response::BestMove(chosen))?;
                }
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

/// Terminal positions report `info depth 0 score mate 0` for checkmate
/// or `info depth 0 score cp 0` for stalemate.
fn search_info<'a>(
    depth: u8,
    result: &SearchResult,
    time_ms: u64,
    pv: &'a [UciMove],
) -> Response<'a> {
    let (depth, nodes) = if result.chosen_move.is_some() {
        (depth, Some(result.nodes))
    } else {
        (0, None)
    };
    let score = match result.score {
        Score::Centipawns(value) => SearchScore::Centipawns(value),
        Score::Mate { plies, winning } => SearchScore::Mate { plies, winning },
    };
    Response::SearchInfo {
        depth,
        score,
        nodes,
        time_ms: nodes.map(|_| time_ms),
        nps: nodes.map(|nodes| nodes_per_second(nodes, time_ms)),
        pv,
    }
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
    fn search_info_uses_elapsed_time_from_the_clock() -> io::Result<()> {
        assert_search_info_time(7250, 3, 68_867_666)
    }

    #[test]
    fn search_info_reports_one_millisecond_for_submillisecond_search() -> io::Result<()> {
        assert_search_info_time(4250, 1, 206_603_000)
    }

    fn assert_search_info_time(end_micros: u64, time: u64, nps: u64) -> io::Result<()> {
        // The starting position visits 206603 nodes (sum of published perft counts).
        let readings = std::cell::Cell::new([4000, end_micros].into_iter());
        let now = || {
            let mut remaining = readings.take();
            let micros = remaining
                .next()
                .expect("one clock reading on each side of minimax");
            readings.set(remaining);
            Duration::from_micros(micros)
        };
        let mut output = Vec::new();
        answer_commands(
            ["go infinite", "stop"].map(|command| Ok(command.to_owned())),
            &mut output,
            now,
        )?;
        assert!(readings.take().next().is_none());
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
