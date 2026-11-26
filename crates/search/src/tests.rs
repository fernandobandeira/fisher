#![cfg(test)]
//! The search that `search` exports, compared with fisher's own plain
//! minimax (fisher-4yn.4, D35). Both use the evaluation private to this crate,
//! so these tests live inside it.

mod comparison;
mod plain_minimax;

#[path = "tests/support/mate_in_two.rs"]
mod mate_in_two;
#[path = "tests/support/perft.rs"]
mod perft;
#[path = "tests/support/principal_variation.rs"]
mod principal_variation;
