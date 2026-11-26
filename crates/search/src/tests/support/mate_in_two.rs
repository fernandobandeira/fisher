#![cfg(test)]
//! Reads the published mate-in-two positions by the parsing rules in
//! `expectations/README.md`. It uses no part of fisher.
//!
//! The unit tests of `search` and its integration tests both include it.

const TABLE: &str = include_str!("../../../../../expectations/mate-in-two.tsv");

/// One data line of `expectations/mate-in-two.tsv`.
pub(super) struct Record {
    /// The case-sensitive Lichess puzzle ID.
    pub(super) id: &'static str,
    /// The full six-field FEN after the first published move.
    pub(super) fen: &'static str,
    /// The published first move of the solution, written as a UCI move.
    pub(super) published_move: &'static str,
}

/// Returns the records in the order of the table, or an error naming the
/// first data line that does not hold exactly three tab-separated fields.
/// Trimming only recognises blank and comment lines. A data line is split
/// with its tabs intact, so a trailing tab counts as a fourth field.
pub(super) fn records() -> Result<Vec<Record>, String> {
    TABLE
        .lines()
        .filter(|line| {
            let line = line.trim();
            !line.is_empty() && !line.starts_with('#')
        })
        .map(record)
        .collect()
}

fn record(line: &'static str) -> Result<Record, String> {
    let mut fields = line.split('\t');
    match (fields.next(), fields.next(), fields.next(), fields.next()) {
        (Some(id), Some(fen), Some(published_move), None) => Ok(Record {
            id,
            fen,
            published_move,
        }),
        _ => Err(format!(
            "expected three tab-separated fields in mate-in-two.tsv: {line:?}"
        )),
    }
}
