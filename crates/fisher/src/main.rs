use std::{
    io::{self, BufRead, Write},
    sync::mpsc,
    thread,
};

use uci::{Command, Input, Response, parse_command};

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
            Input::Handled(Command::Quit) => break,
            Input::Ignored | Input::Unknown => {}
        }
    }
    Ok(())
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
