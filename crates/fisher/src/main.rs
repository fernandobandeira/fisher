use std::{
    io::{self, BufRead, Write},
    sync::mpsc,
    thread,
};

use rules::Position;
use search::{Score, minimax};
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

    answer_commands(lines, &mut io::stdout().lock())
}

fn answer_commands(
    lines: impl IntoIterator<Item = io::Result<String>>,
    output: &mut impl Write,
) -> io::Result<()> {
    let mut position = Position::starting();
    let mut held_move = None;
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
                // TODO(fisher-2aj.5): Answer isready, stop and quit while a search runs.
                const DEPTH: u8 = 4;
                let (chosen, score, nodes) = minimax(&position, DEPTH);
                let score = match score {
                    Score::Centipawns(value) => SearchScore::Centipawns(value),
                    Score::Mate { plies, winning } => {
                        let moves = i32::from(plies.div_ceil(2));
                        SearchScore::Mate(if winning { moves } else { -moves })
                    }
                };
                send(
                    output,
                    &Response::SearchInfo {
                        depth: if chosen.is_some() { DEPTH } else { 0 },
                        score,
                        nodes: chosen.map(|_| nodes),
                    },
                )?;
                let chosen = chosen.map_or(UciMove::Null, UciMove::from);
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
