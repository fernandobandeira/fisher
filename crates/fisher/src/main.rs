use std::{
    io::{self, BufRead, Write},
    sync::mpsc,
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use rules::{Move, Position};
use uci::{Command, Input, PositionCommand, PositionSource, Response, UciMove, parse_command};

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
    let mut random = RandomMove::from_clock();
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
            Input::Handled(Command::Position(command)) => {
                if let Some(next) = command.ok().and_then(setup_position) {
                    position = next;
                } else {
                    send(output, &Response::InfoString("position rejected"))?;
                }
            }
            Input::Handled(Command::Go(_)) => {
                // TODO(fisher-7vo): Obey the supplied search limits.
                // TODO(fisher-2ha): Wait for ponderhit when pondering.
                let moves = position.legal_moves();
                let chosen = random
                    .choose(&moves)
                    .copied()
                    .map_or(UciMove::Null, UciMove::from);
                send(output, &Response::BestMove(chosen))?;
            }
            Input::Handled(Command::Quit) => break,
            Input::Handled(Command::UciNewGame) | Input::Ignored | Input::Unknown => {}
        }
    }
    Ok(())
}

fn setup_position(command: PositionCommand) -> Option<Position> {
    let mut position = match command.source {
        PositionSource::Startpos => Position::starting(),
        PositionSource::Fen(fen) => Position::from_fen(&fen).ok()?,
    };
    for uci_move in command.moves {
        let UciMove::Normal {
            from,
            to,
            promotion,
        } = uci_move
        else {
            return None;
        };
        position = position.legal_move(from, to, promotion)?.play();
    }
    Some(position)
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

struct RandomMove(u64);

impl RandomMove {
    fn from_clock() -> Self {
        let elapsed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        Self(elapsed.as_secs() ^ u64::from(elapsed.subsec_nanos()))
    }

    fn choose<'a>(&mut self, moves: &'a [Move]) -> Option<&'a Move> {
        // TODO(fisher-2aj): Choose moves by search and evaluation and report search info.
        let count = u64::try_from(moves.len()).ok()?;
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.0;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        let index = usize::try_from((value ^ (value >> 31)).checked_rem(count)?).ok()?;
        moves.get(index)
    }
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
