#![cfg(test)]
//! The positions where a transcript in `expectations/` leaves the score open
//! with `{score}`. The transcripts cannot tell whether such a score is the
//! right one, so the tests of `search` compare the search with plain minimax
//! at each of these positions, at every depth from 1 to `DEEPEST`.
//!
//! The transcript runner in `crates/fisher/tests` includes this file too. It
//! fails on a `{score}` at a depth from 1 to `DEEPEST` from a position missing
//! here. Each FEN is written as shakmaty writes it, with the en passant square
//! only when a capture there is legal.

/// The deepest `{score}` the transcript runner checks.
pub(super) const DEEPEST: u8 = 4;

pub(super) const POSITIONS: &[&str] = &[
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1",
    "rnbqk2r/1pp1bppp/p2Ppn2/8/8/5N2/PPPPBPPP/RNBQ1RK1 b kq - 5 6",
    "3Q4/k7/1N6/3K4/8/1R6/8/8 b - - 2 2",
    "8/8/3P1B2/6K1/4k3/8/5Q2/4N3 b - - 0 1",
    "7k/8/5K1Q/8/8/8/8/8 b - - 0 1",
];
