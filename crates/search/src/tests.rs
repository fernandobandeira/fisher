#![cfg(test)]
//! The search that `search` exports, compared with fisher's own plain
//! minimax (fisher-4yn.4, D35). Both use the evaluation private to this crate,
//! so these tests live inside it.

mod comparison;
mod move_order_memory;
mod plain_minimax;
mod remembered_move;
mod several_variations;
mod tied_root_moves;

#[path = "tests/support/mate_in_two.rs"]
mod mate_in_two;
#[path = "tests/support/perft.rs"]
mod perft;
#[path = "tests/support/principal_variation.rs"]
mod principal_variation;
