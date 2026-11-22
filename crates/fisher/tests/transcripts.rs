use std::{
    fs,
    io::{self, BufRead, BufReader, Write},
    path::Path,
    process::{Child, Command, Stdio},
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};

use shakmaty::{CastlingMode, Chess, Move, Position, fen::Fen, uci::UciMove};

const TIMEOUT: Duration = Duration::from_secs(5);

#[test]
fn handshake() -> io::Result<()> {
    play_transcript("handshake")
}

#[test]
fn unknown_input() -> io::Result<()> {
    play_transcript("unknown-input")
}

#[test]
fn ignored_commands() -> io::Result<()> {
    play_transcript("ignored-commands")
}

#[test]
fn quit() -> io::Result<()> {
    play_transcript("quit")
}

#[test]
fn end_of_input() -> io::Result<()> {
    play_transcript("end-of-input")
}

#[test]
fn startpos() -> io::Result<()> {
    play_transcript("startpos")
}

#[test]
fn new_game() -> io::Result<()> {
    play_transcript("new-game")
}

#[test]
fn position_moves() -> io::Result<()> {
    play_transcript("position-moves")
}

#[test]
fn go_parameters() -> io::Result<()> {
    play_transcript("go-parameters")
}

#[test]
fn terminal_positions() -> io::Result<()> {
    play_transcript("terminal-positions")
}

#[test]
fn halfmove_clock() -> io::Result<()> {
    play_transcript("halfmove-clock")
}

fn play_transcript(name: &str) -> io::Result<()> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../expectations/uci")
        .join(format!("{name}.uci"));
    let transcript = fs::read_to_string(path)?;
    let mut session = Session::start()?;
    let mut ended = false;

    for (line_index, line) in transcript.lines().enumerate() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        ended = if ended {
            Err(io::Error::other("record after ! exit"))
        } else {
            session.play_record(line)
        }
        .map_err(|error| io::Error::other(format!("{name}.uci:{}: {error}", line_index + 1)))?;
    }

    if ended {
        Ok(())
    } else {
        Err(io::Error::other(format!("{name}.uci: missing ! exit")))
    }
}

fn decode_command(command: &str) -> io::Result<Vec<u8>> {
    let mut decoded = Vec::with_capacity(command.len());
    let mut bytes = command.bytes();
    while let Some(byte) = bytes.next() {
        decoded.push(if byte == b'\\' {
            match bytes.next() {
                Some(b't') => b'\t',
                Some(b'r') => b'\r',
                Some(b'\\') => b'\\',
                Some(b'x') => {
                    let high = decode_hex_digit(bytes.next())?;
                    let low = decode_hex_digit(bytes.next())?;
                    high * 16 + low
                }
                other => {
                    return Err(io::Error::other(format!(
                        "invalid escape {:?}",
                        other.map(char::from)
                    )));
                }
            }
        } else {
            byte
        });
    }
    Ok(decoded)
}

fn decode_hex_digit(byte: Option<u8>) -> io::Result<u8> {
    match byte {
        Some(byte @ b'0'..=b'9') => Ok(byte - b'0'),
        Some(byte @ b'a'..=b'f') => Ok(byte - b'a' + 10),
        Some(byte @ b'A'..=b'F') => Ok(byte - b'A' + 10),
        _ => Err(io::Error::other("invalid hex escape")),
    }
}

fn reference_command<'a>(tokens: &mut impl Iterator<Item = &'a str>) -> Option<&'a str> {
    tokens.find(|&token| {
        matches!(
            token,
            "uci"
                | "isready"
                | "ucinewgame"
                | "position"
                | "go"
                | "quit"
                | "setoption"
                | "debug"
                | "ponderhit"
                | "register"
        )
    })
}

fn reference_position<'a>(mut tokens: impl Iterator<Item = &'a str>) -> Option<Chess> {
    let mut position = match tokens.next()? {
        "startpos" => Chess::default(),
        "fen" => {
            let fields: Vec<_> = tokens.by_ref().take(6).collect();
            if fields.len() != 6 {
                return None;
            }
            fields
                .join(" ")
                .parse::<Fen>()
                .ok()?
                .into_position(CastlingMode::Standard)
                .ok()?
        }
        _ => return None,
    };
    match tokens.next() {
        None => return Some(position),
        Some("moves") => {}
        Some(_) => return None,
    }
    for token in tokens {
        let movement = reference_move(&position, token)?;
        position.play_unchecked(movement);
    }
    Some(position)
}

fn reference_move(position: &Chess, text: &str) -> Option<Move> {
    position
        .legal_moves()
        .into_iter()
        .find(|&movement| UciMove::from_standard(movement).to_string() == text)
}

fn reply_matches(expected: &str, actual: &str, position: &Chess) -> bool {
    match expected {
        "id name fisher {version}" => {
            actual == concat!("id name fisher ", env!("CARGO_PKG_VERSION"))
        }
        "bestmove {legal}" => actual
            .strip_prefix("bestmove ")
            .is_some_and(|text| reference_move(position, text).is_some()),
        "info string {message}" => actual.starts_with("info string "),
        _ => expected == actual,
    }
}

struct Session {
    child: Child,
    output: Receiver<io::Result<String>>,
    position: Chess,
}

impl Session {
    fn start() -> io::Result<Self> {
        let child = Command::new(env!("CARGO_BIN_EXE_fisher"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let (sender, output) = mpsc::channel();
        let mut session = Self {
            child,
            output,
            position: Chess::default(),
        };
        let stdout = session
            .child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("missing fisher stdout"))?;

        thread::Builder::new().spawn(move || {
            for line in BufReader::new(stdout).lines() {
                if sender.send(line).is_err() {
                    break;
                }
            }
        })?;

        Ok(session)
    }

    fn play_record(&mut self, line: &str) -> io::Result<bool> {
        if let Some(command) = line.strip_prefix("> ") {
            self.send(command, b'\n')?;
        } else if let Some(command) = line.strip_prefix(">CR ") {
            self.send(command, b'\r')?;
        } else if let Some(expected) = line.strip_prefix("< ") {
            self.expect_reply(expected)?;
        } else if line == "! eof" {
            drop(self.child.stdin.take());
        } else if line == "! exit" {
            self.expect_exit()?;
            return Ok(true);
        } else {
            return Err(io::Error::other(format!("unknown record {line:?}")));
        }
        Ok(false)
    }

    fn send(&mut self, command: &str, ending: u8) -> io::Result<()> {
        let stdin = self
            .child
            .stdin
            .as_mut()
            .ok_or_else(|| io::Error::other("fisher stdin is closed"))?;
        let command = decode_command(command)?;
        stdin.write_all(&command)?;
        stdin.write_all(&[ending])?;
        stdin.flush()?;
        let text = String::from_utf8_lossy(&command);
        let mut tokens = text.split_whitespace();
        if reference_command(&mut tokens) == Some("position")
            && let Some(position) = reference_position(tokens)
        {
            self.position = position;
        }
        Ok(())
    }

    fn expect_reply(&self, expected: &str) -> io::Result<()> {
        let actual = self.next_reply()?.ok_or_else(|| {
            io::Error::other(format!("stdout closed while expecting {expected:?}"))
        })?;
        if reply_matches(expected, &actual, &self.position) {
            Ok(())
        } else {
            Err(io::Error::other(format!(
                "expected {expected:?}, got {actual:?}"
            )))
        }
    }

    fn next_reply(&self) -> io::Result<Option<String>> {
        match self.output.recv_timeout(TIMEOUT) {
            Ok(line) => line.map(Some),
            Err(mpsc::RecvTimeoutError::Disconnected) => Ok(None),
            Err(error @ mpsc::RecvTimeoutError::Timeout) => Err(io::Error::other(error)),
        }
    }

    fn expect_exit(&mut self) -> io::Result<()> {
        if let Some(extra) = self.next_reply()? {
            return Err(io::Error::other(format!("unexpected output {extra:?}")));
        }
        let deadline = Instant::now() + TIMEOUT;
        loop {
            if let Some(status) = self.child.try_wait()? {
                return if status.success() {
                    Ok(())
                } else {
                    Err(io::Error::other(format!("fisher exited with {status}")))
                };
            }
            if Instant::now() >= deadline {
                return Err(io::Error::other("timed out waiting for fisher to exit"));
            }
            thread::sleep(Duration::from_millis(5));
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        drop(self.child.kill());
        drop(self.child.wait());
    }
}
