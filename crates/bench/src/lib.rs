use std::{
    ffi::OsStr,
    io::{self, Write},
};

use rules::Position;
use search::minimax;

const POSITIONS: &str = include_str!("../../../expectations/perft.epd");
const DEPTH: u8 = 4;

/// Runs the bench for the first command-line argument `bench` and returns whether it matched.
///
/// # Errors
///
/// Returns an error if a bench position cannot be parsed or writing the report fails.
pub fn run_if_requested(argument: Option<&OsStr>, output: &mut impl Write) -> io::Result<bool> {
    if argument != Some(OsStr::new("bench")) {
        return Ok(false);
    }

    let positions = POSITIONS
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'));
    let mut total = 0;
    for (index, line) in positions.enumerate() {
        let fen = line.split(';').next().unwrap_or_default().trim();
        let position = Position::from_fen(fen).map_err(io::Error::other)?;
        let nodes = minimax(&position, DEPTH).nodes;
        writeln!(output, "Position {}: {nodes}", index + 1)?;
        total += nodes;
    }
    writeln!(output, "Nodes searched  : {total}")?;
    output.flush()?;
    Ok(true)
}
