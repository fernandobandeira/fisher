#![cfg(test)]
//! Reads the published perft counts in `expectations/perft.epd`. It uses no
//! part of fisher.

use std::error::Error;

const PERFT: &str = include_str!("../../../../../expectations/perft.epd");

/// One data line of `expectations/perft.epd`.
pub(super) struct Record {
    pub(super) fen: &'static str,
    counts: &'static str,
}

/// Returns the positions in the order of the file.
pub(super) fn records() -> Result<Vec<Record>, Box<dyn Error>> {
    PERFT
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let (fen, counts) = line.split_once(';').ok_or("missing perft counts")?;
            Ok(Record {
                fen: fen.trim(),
                counts,
            })
        })
        .collect()
}

/// Checks that `nodes_at_depth` gives the sum of the published counts at
/// plies 1 to `depth` for every position.
pub(super) fn check_perft(
    depth: u8,
    nodes_at_depth: impl Fn(&str, u8) -> Result<u64, Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
    for record in records()? {
        let mut expected = 0;
        for ply in 1..=depth {
            let label = format!("D{ply} ");
            let count = record
                .counts
                .split(';')
                .find_map(|entry| entry.trim().strip_prefix(&label))
                .ok_or_else(|| format!("missing perft count for {} at ply {ply}", record.fen))?;
            expected += count.parse::<u64>()?;
        }
        assert_eq!(
            nodes_at_depth(record.fen, depth)?,
            expected,
            "{} at depth {depth}",
            record.fen
        );
    }
    Ok(())
}
