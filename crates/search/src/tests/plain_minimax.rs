#![cfg(test)]
//! Plain minimax, copied from `search` as it stood at 06de6a6, before the
//! search pruned (D35). It supplies the expected score and root move for the
//! search that `search` exports.
//!
//! `minimax`, `search_depth`, `mate_within_depth_or_centipawns` and `search`
//! are the copy. They keep their own mate constant, so a change to the
//! constants of the exported search does not change them. They share the
//! evaluation, `material`, with the exported search. `plain_minimax` is new:
//! it plays each root move with the copied `search` to score every root move.

use std::{cmp::Reverse, convert::Infallible};

use rules::{Move, Position};

use crate::{Score, SearchResult, material};

const CHECKMATE: i32 = 100_000;

/// What plain minimax finds at the root for one depth.
pub(super) struct PlainMinimax {
    /// The score of the root, from the root's color.
    pub(super) score: Score,
    /// Every root move in legal order, with its score from the root's color.
    /// The distance to checkmate counts the root move's own ply.
    /// Empty at depth zero and at a root with no legal move.
    pub(super) root_moves: Vec<(Move, Score)>,
    /// Every move played, counted by rule 7 of fisher-2aj.
    pub(super) nodes: u64,
}

impl PlainMinimax {
    /// The first root move in legal order among those scored as best.
    pub(super) fn chosen_root_move(&self) -> Option<Move> {
        self.root_moves
            .iter()
            .find(|(_, score)| *score == self.score)
            .map(|(movement, _)| *movement)
    }
}

/// Searches every legal move to `depth` plies and scores every root move.
pub(super) fn plain_minimax(position: &Position, depth: u8) -> PlainMinimax {
    let mut root_moves = Vec::new();
    let nodes = visit_root_moves(position, depth, |movement, score, _| {
        root_moves.push((movement, score));
    });
    let best = root_moves
        .iter()
        .map(|&(_, score)| score)
        .max_by_key(|&score| order(score));
    PlainMinimax {
        // At depth zero or a terminal root, score the position directly.
        score: best.unwrap_or_else(|| minimax(position, depth).score),
        root_moves,
        nodes,
    }
}

/// Supplies a search closure for testing the several-variations corpus driver.
/// Stable sorting keeps the legal order among tied root moves.
pub(super) fn scored_variations(
    position: &Position,
    depth: u8,
    requested: u8,
) -> Vec<(Score, Vec<Move>)> {
    let mut variations = Vec::new();
    visit_root_moves(position, depth, |movement, score, replies| {
        let mut moves = Vec::with_capacity(replies.len() + 1);
        moves.push(movement);
        moves.extend_from_slice(replies);
        variations.push((score, moves));
    });
    variations.sort_by_key(|&(score, _)| Reverse(order(score)));
    variations.truncate(usize::from(requested));
    variations
}

fn visit_root_moves(
    position: &Position,
    depth: u8,
    mut visit: impl FnMut(Move, Score, &[Move]),
) -> u64 {
    let Some(reply_depth) = depth.checked_sub(1) else {
        return 0;
    };
    let mut nodes = 0;
    let mut variations: Vec<_> = (0..reply_depth).map(|_| Vec::new()).collect();
    for movement in position.legal_moves() {
        nodes += 1;
        let Ok(score) = search(
            &movement.play(),
            1,
            &mut nodes,
            &mut variations,
            &mut || Ok::<(), Infallible>(()),
        );
        let replies = variations.first().map_or(&[][..], Vec::as_slice);
        visit(
            movement,
            mate_within_depth_or_centipawns(-score, depth),
            replies,
        );
    }
    nodes
}

/// Ranks a score from the root's color: the higher, the better.
fn order(score: Score) -> i32 {
    match score {
        Score::Centipawns(value) => value,
        Score::Mate {
            plies,
            winning: true,
        } => CHECKMATE - i32::from(plies),
        Score::Mate {
            plies,
            winning: false,
        } => i32::from(plies) - CHECKMATE,
    }
}

// The copy starts here.

/// Searches every legal move to the given depth in plies.
/// At depth zero the result has no best move, even if legal moves exist.
/// The score is from the root's side to move, and nodes exclude the root.
fn minimax(position: &Position, depth: u8) -> SearchResult {
    let Ok(result) = search_depth(position, depth, &mut || Ok::<(), Infallible>(()));
    result
}

fn search_depth<E>(
    position: &Position,
    depth: u8,
    before_node: &mut impl FnMut() -> Result<(), E>,
) -> Result<SearchResult, E> {
    let mut nodes = 0;
    // Reuse one buffer per ply across sibling searches.
    let mut variations: Vec<_> = (0..depth).map(|_| Vec::new()).collect();
    search(position, 0, &mut nodes, &mut variations, before_node).map(|score| {
        let principal_variation = variations.into_iter().next().unwrap_or_default();
        SearchResult {
            best_move: principal_variation.first().copied(),
            score: mate_within_depth_or_centipawns(score, depth),
            nodes,
            principal_variation,
        }
    })
}

/// Returns a mate score if the mate is within the search depth, or a centipawn score otherwise.
fn mate_within_depth_or_centipawns(score: i32, depth: u8) -> Score {
    let mate_plies = CHECKMATE - score.abs();
    if let Ok(plies) = u8::try_from(mate_plies)
        && plies <= depth
    {
        Score::Mate {
            plies,
            winning: score > 0,
        }
    } else {
        Score::Centipawns(score)
    }
}

fn search<E>(
    position: &Position,
    ply: u8,
    nodes: &mut u64,
    variations: &mut [Vec<Move>],
    before_node: &mut impl FnMut() -> Result<(), E>,
) -> Result<i32, E> {
    let Some((best_variation, replies)) = variations.split_first_mut() else {
        return Ok(material(position));
    };
    best_variation.clear();
    let mut best_score = i32::MIN;
    position
        .legal_moves()
        .into_iter()
        .try_for_each(|movement| {
            before_node().and_then(|()| {
                *nodes += 1;
                search(&movement.play(), ply + 1, nodes, replies, before_node).map(|score| {
                    let score = -score;
                    if score > best_score {
                        best_score = score;
                        best_variation.clear();
                        best_variation.push(movement);
                        if let Some(reply) = replies.first() {
                            best_variation.extend_from_slice(reply);
                        }
                    }
                })
            })
        })
        .map(|()| {
            if best_variation.is_empty() {
                if position.is_in_check() {
                    i32::from(ply) - CHECKMATE
                } else {
                    0
                }
            } else {
                best_score
            }
        })
}

// The copy ends here.

#[test]
fn node_count_matches_published_perft_at_two_plies() -> Result<(), Box<dyn std::error::Error>> {
    super::perft::check_perft(2, |fen, depth| {
        Ok(plain_minimax(&Position::from_fen(fen)?, depth).nodes)
    })
}

#[test]
#[ignore = "run in release mode for the published four-ply counts"]
fn node_count_matches_published_perft_at_four_plies() -> Result<(), Box<dyn std::error::Error>> {
    super::perft::check_perft(4, |fen, depth| {
        Ok(plain_minimax(&Position::from_fen(fen)?, depth).nodes)
    })
}
