use std::{
    ffi::OsStr,
    io::{self, Write},
    num::NonZeroU8,
    sync::{Arc, atomic::AtomicBool},
    time::Instant,
};

use deepening::{Limits, SearchReport, SearchRequest, search};
use rules::Position;

const POSITIONS: &str = include_str!("../../../expectations/perft.epd");
const DEPTH: u64 = 5;

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
        let request = SearchRequest {
            position,
            limits: Limits {
                depth: Some(DEPTH),
                ..Limits::default()
            },
            requested_variations: NonZeroU8::MIN,
            interrupt: Arc::new(AtomicBool::new(false)),
            started: Instant::now(),
        };
        let mut nodes = 0;
        search(request, &mut |report| {
            if let SearchReport::FinishedDepth(finished) = report {
                nodes = finished.nodes;
            }
        });
        writeln!(output, "Position {}: {nodes}", index + 1)?;
        total += nodes;
    }
    writeln!(output, "Nodes searched  : {total}")?;
    output.flush()?;
    Ok(true)
}
