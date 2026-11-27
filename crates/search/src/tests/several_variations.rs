#![cfg(test)]
//! The D13 and D48 checks for fisher-8hc.9. The tests here supply
//! hand-written variations before search can return several of them.

use std::{cmp::Reverse, error::Error, fmt::Display};

use rules::{Move, Position};
use shakmaty::{CastlingMode, Chess, fen::Fen};

use super::{
    comparison::{check_each_prefix, move_text, score_value, visit_comparison_corpus},
    plain_minimax::{plain_minimax, scored_variations},
    principal_variation,
};
use crate::Score;

type TestResult = Result<(), Box<dyn Error>>;
pub(super) type ScoredVariation<'a> = (Score, &'a [Move]);

/// Checks the same positions and depths as the every-commit comparison.
/// The closure searches without interruption. At a terminal root it must
/// return `Some(vec![])`, a finished result with no variations.
/// The existing setting-1 search's empty principal variation maps to
/// no first variation. A closure returning `None` failed to finish.
pub(super) fn check_comparison_corpus(
    mut search: impl FnMut(&Position, u8, u8) -> Option<Vec<(Score, Vec<Move>)>>,
) -> TestResult {
    visit_comparison_corpus(|root, depth| {
        let single = crate::minimax(root, depth);
        let single_moves = single.principal_variation.as_slice();
        let first = (!single_moves.is_empty()).then_some((single.score, single_moves));
        for requested in [2, 3, 255] {
            let case = || format!("{} at depth {depth}, setting {requested}", root.to_fen());
            let result = search(root, depth, requested)
                .ok_or_else(|| format!("{}: expected a finished result", case()))?;
            let variations: Vec<_> = result
                .iter()
                .map(|(score, moves)| (*score, moves.as_slice()))
                .collect();
            check_variations(root, depth, requested, &variations)
                .and_then(|()| check_first_variation(&variations, first))
                .map_err(|error| format!("{}: {error}", case()))?;
        }
        Ok(())
    })
}

/// Checks the count, distinct root moves, exact scores, rank, D5 replay and
/// the actual moves and scores of every prefix.
/// Plain minimax scores each root move with its own ply included in the mate
/// distance. Only scores are ranked here, so any permutation of tied moves
/// passes if each variation earns its score.
pub(super) fn check_variations(
    root: &Position,
    depth: u8,
    requested: u8,
    variations: &[ScoredVariation<'_>],
) -> Result<(), String> {
    let expected = plain_minimax(root, depth);
    let count = usize::from(requested).min(expected.root_moves.len());
    let actual = variations.len();
    if actual != count {
        return Err(format!("expected {count} variations, got {actual}"));
    }
    if variations.is_empty() {
        return Ok(());
    }
    let mut ranked_scores: Vec<_> = expected
        .root_moves
        .iter()
        .map(|&(_, score)| score)
        .collect();
    ranked_scores.sort_unstable_by_key(|&score| Reverse(score_value(score)));
    let chess = root
        .to_fen()
        .parse::<Fen>()
        .map_err(|error| error.to_string())?
        .into_position::<Chess>(CastlingMode::Standard)
        .map_err(|error| error.to_string())?;
    let mut seen = Vec::with_capacity(count);
    for (index, (&(score, moves), ranked_score)) in variations.iter().zip(ranked_scores).enumerate()
    {
        let number = index + 1;
        let root_move = moves
            .first()
            .ok_or_else(|| format!("variation {number} has no root move"))?;
        check_root_move(root_move, score, &expected.root_moves, &mut seen)
            .map_err(|error| format!("variation {number}: {error}"))?;
        if score != ranked_score {
            return Err(format!(
                "variation {number}: expected score {ranked_score:?} at this rank, got {score:?}"
            ));
        }
        check_variation(root, &chess, depth, score, moves)
            .map_err(|error| format!("variation {number}: {error}"))?;
    }
    Ok(())
}

fn check_variation(
    root: &Position,
    chess: &Chess,
    depth: u8,
    score: Score,
    moves: &[Move],
) -> Result<(), String> {
    principal_variation::verify(
        chess,
        depth,
        score_value(score),
        moves.iter().copied().map(move_text),
    )?;
    check_each_prefix(root, depth, score, moves)
}

fn check_root_move(
    root_move: &Move,
    score: Score,
    root_moves: &[(Move, Score)],
    seen: &mut Vec<Move>,
) -> Result<(), String> {
    let text = move_text(*root_move);
    let (_, exact) = root_moves
        .iter()
        .find(|(movement, _)| movement == root_move)
        .ok_or_else(|| format!("{text} is not a root move"))?;
    if seen.contains(root_move) {
        return Err(format!("repeated root move {text}"));
    }
    if score != *exact {
        return Err(format!(
            "plain minimax scores {text} as {exact:?}, got {score:?}"
        ));
    }
    seen.push(*root_move);
    Ok(())
}

/// Compares the first variation with the setting-1 score and all its moves.
/// A root with no legal move has no variation at either setting.
pub(super) fn check_first_variation(
    variations: &[ScoredVariation<'_>],
    single: Option<ScoredVariation<'_>>,
) -> Result<(), String> {
    if variations.first().copied() == single {
        Ok(())
    } else {
        Err("first variation differs from the result at setting 1".to_owned())
    }
}

const MATE_IN_ONE: &str = "7k/8/5KQ1/8/8/8/8/8 w - - 0 1";
const EVEN: Score = Score::Centipawns(0);
const WIN_IN_ONE: Score = Score::Mate {
    plies: 1,
    winning: true,
};

fn play<const N: usize>(root: &Position, texts: [&str; N]) -> Result<[Move; N], String> {
    let mut position = *root;
    let moves: Vec<_> = texts
        .into_iter()
        .map(|text| {
            let movement = position
                .legal_moves()
                .into_iter()
                .find(|&movement| move_text(movement) == text)
                .ok_or_else(|| format!("{text} is not legal in {}", position.to_fen()))?;
            position = movement.play();
            Ok(movement)
        })
        .collect::<Result<_, String>>()?;
    moves.try_into().map_err(|_| format!("expected {N} moves"))
}

fn assert_rejected(result: Result<(), impl Display>, reason: &str) -> TestResult {
    let error = result
        .err()
        .ok_or_else(|| format!("accepted: {reason}"))?
        .to_string();
    assert!(error.contains(reason), "expected {reason:?}, got {error}");
    Ok(())
}

#[test]
fn accepts_three_tied_root_moves_in_either_order() -> TestResult {
    let root = Position::starting();
    let e4 = play(&root, ["e2e4"])?;
    let d4 = play(&root, ["d2d4"])?;
    let c4 = play(&root, ["c2c4"])?;
    let mut variations: [ScoredVariation<'_>; _] = [(EVEN, &e4), (EVEN, &d4), (EVEN, &c4)];
    check_variations(&root, 1, 3, &variations)?;
    variations.reverse();
    check_variations(&root, 1, 3, &variations)?;
    Ok(())
}

#[test]
fn accepts_mate_before_centipawns_and_rejects_the_reversed_order() -> TestResult {
    let root = Position::from_fen(MATE_IN_ONE)?;
    let mate = play(&root, ["g6g7"])?;
    let material = play(&root, ["g6h5", "h8g8"])?;
    let mate = (WIN_IN_ONE, mate.as_slice());
    let material = (Score::Centipawns(900), material.as_slice());
    check_variations(&root, 2, 2, &[mate, material])?;
    assert_rejected(
        check_variations(&root, 2, 2, &[material, mate]),
        "expected score Mate { plies: 1, winning: true } at this rank",
    )?;
    assert_rejected(
        check_variations(&root, 2, 1, &[material]),
        "expected score Mate { plies: 1, winning: true } at this rank",
    )
}

#[test]
fn rejects_a_slower_win_before_a_faster_win() -> TestResult {
    let root = Position::from_fen(MATE_IN_ONE)?;
    let immediate = play(&root, ["g6g7"])?;
    let delayed = play(&root, ["g6g5", "h8h7", "g5g7"])?;
    let immediate = (WIN_IN_ONE, immediate.as_slice());
    let delayed = (
        Score::Mate {
            plies: 3,
            winning: true,
        },
        delayed.as_slice(),
    );
    check_variations(&root, 4, 2, &[immediate, delayed])?;
    assert_rejected(
        check_variations(&root, 4, 2, &[delayed, immediate]),
        "expected score Mate { plies: 1, winning: true } at this rank, got Mate { plies: 3, winning: true }",
    )
}

#[test]
fn rejects_a_faster_loss_before_a_slower_loss() -> TestResult {
    let root = Position::from_fen("1k6/8/K7/4Q3/8/8/8/8 b - - 0 1")?;
    let immediate = play(&root, ["b8a8", "e5h8"])?;
    let delayed = play(&root, ["b8c8", "e5e7", "c8b8", "e7d8"])?;
    let immediate = (
        Score::Mate {
            plies: 2,
            winning: false,
        },
        immediate.as_slice(),
    );
    let delayed = (
        Score::Mate {
            plies: 4,
            winning: false,
        },
        delayed.as_slice(),
    );
    check_variations(&root, 5, 2, &[delayed, immediate])?;
    assert_rejected(
        check_variations(&root, 5, 2, &[immediate, delayed]),
        "expected score Mate { plies: 4, winning: false } at this rank, got Mate { plies: 2, winning: false }",
    )
}

#[test]
fn rejects_the_wrong_count_for_the_setting_or_root() -> TestResult {
    let root = Position::starting();
    let e4 = play(&root, ["e2e4"])?;
    let d4 = play(&root, ["d2d4"])?;
    let variations: [ScoredVariation<'_>; _] = [(EVEN, &e4), (EVEN, &d4)];
    assert_rejected(
        check_variations(&root, 1, 3, &variations),
        "expected 3 variations, got 2",
    )?;
    assert_rejected(
        check_variations(&root, 1, 1, &variations),
        "expected 1 variations, got 2",
    )?;

    // The only legal move, and its depth-1 score, are pinned by go-infinite.uci.
    let forced = Position::from_fen("7k/8/5K1Q/8/8/8/8/8 b - - 0 1")?;
    let escape = play(&forced, ["h8g8"])?;
    check_variations(&forced, 1, 255, &[(Score::Centipawns(-900), &escape)])?;
    assert_rejected(
        check_variations(&forced, 1, 255, &[]),
        "expected 1 variations, got 0",
    )
}

#[test]
fn rejects_a_repeated_missing_or_illegal_root_move() -> TestResult {
    let root = Position::starting();
    let [e4, e5] = play(&root, ["e2e4", "e7e5"])?;
    assert_rejected(
        check_variations(&root, 1, 2, &[(EVEN, &[e4]), (EVEN, &[e4])]),
        "repeated root move e2e4",
    )?;
    assert_rejected(
        check_variations(&root, 1, 1, &[(EVEN, &[])]),
        "has no root move",
    )?;
    assert_rejected(
        check_variations(&root, 1, 1, &[(EVEN, &[e5])]),
        "e7e5 is not a root move",
    )
}

#[test]
fn rejects_wrong_centipawns_mate_distance_and_mate_winner() -> TestResult {
    let root = Position::starting();
    let e4 = play(&root, ["e2e4"])?;
    assert_rejected(
        check_variations(&root, 1, 1, &[(Score::Centipawns(100), &e4)]),
        "plain minimax scores e2e4 as Centipawns(0), got Centipawns(100)",
    )?;

    let root = Position::from_fen(MATE_IN_ONE)?;
    let mate = play(&root, ["g6g7"])?;
    for score in [
        Score::Mate {
            plies: 2,
            winning: true,
        },
        Score::Mate {
            plies: 1,
            winning: false,
        },
    ] {
        assert_rejected(
            check_variations(&root, 2, 1, &[(score, &mate)]),
            &format!("plain minimax scores g6g7 as {WIN_IN_ONE:?}, got {score:?}"),
        )?;
    }
    Ok(())
}

#[test]
fn rejects_illegal_and_wrong_length_variations() -> TestResult {
    let root = Position::starting();
    let [e4, e5] = play(&root, ["e2e4", "e7e5"])?;
    assert_rejected(
        check_variations(&root, 2, 1, &[(EVEN, &[e4, e4])]),
        "illegal principal variation move e2e4 after 1 plies",
    )?;
    assert_rejected(
        check_variations(&root, 2, 1, &[(EVEN, &[e4])]),
        "principal variation expected 2 plies, got 1",
    )?;
    assert_rejected(
        check_variations(&root, 1, 1, &[(EVEN, &[e4, e5])]),
        "principal variation exceeds depth 1",
    )
}

#[test]
fn rejects_an_exact_root_score_that_the_moves_do_not_support() -> TestResult {
    // After Rxa7, ...Nxa7 leaves Black a knight up. ...Kg8 leaves White
    // a rook against that knight instead, so it cannot explain cp -300.
    let root = Position::from_fen("7k/q7/8/1n6/8/8/8/R6K w - - 0 1")?;
    let recapture = play(&root, ["a1a7", "b5a7"])?;
    let king_move = play(&root, ["a1a7", "h8g8"])?;
    let score = Score::Centipawns(-300);
    check_variations(&root, 2, 1, &[(score, &recapture)])?;
    assert_rejected(
        check_variations(&root, 2, 1, &[(score, &king_move)]),
        "ends with material 200, not the score -300",
    )
}

#[test]
fn rejects_a_reply_copied_from_a_different_variation() -> TestResult {
    let root = Position::starting();
    let d4 = play(&root, ["d2d4", "e7e5"])?;
    let [_, e5_after_d4] = d4;
    let [e4] = play(&root, ["e2e4"])?;
    assert_rejected(
        check_variations(&root, 2, 2, &[(EVEN, &d4), (EVEN, &[e4, e5_after_d4])]),
        "e7e5 is not legal after 1 plies",
    )
}

#[test]
fn rejects_a_later_variation_with_a_worse_reply() -> TestResult {
    let root =
        Position::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")?;
    let single = crate::minimax(&root, 3);
    let worse_reply = play(&root, ["d5d6", "h3g2", "e2a6"])?;
    assert_rejected(
        check_variations(
            &root,
            3,
            2,
            &[
                (single.score, &single.principal_variation),
                (Score::Centipawns(200), &worse_reply),
            ],
        ),
        concat!(
            "after h3g2, 2 plies into the principal variation, ",
            "plain minimax scores Centipawns(800), not Centipawns(200)"
        ),
    )
}

#[test]
fn accepts_no_variation_at_a_terminal_root() -> TestResult {
    for fen in [
        "7k/6Q1/6K1/8/8/8/8/8 b - - 0 1",
        "7k/5Q2/6K1/8/8/8/8/8 b - - 0 1",
    ] {
        let root = Position::from_fen(fen)?;
        check_variations(&root, 2, 255, &[])?;
        check_first_variation(&[], None)?;
        assert_rejected(
            check_variations(&root, 2, 255, &[(EVEN, &[])]),
            "expected 0 variations, got 1",
        )?;
    }
    Ok(())
}

#[test]
fn first_variation_must_match_the_single_score_and_all_moves() -> TestResult {
    let root = Position::starting();
    let single = play(&root, ["e2e4", "e7e5"])?;
    let other_root_move = play(&root, ["d2d4", "d7d5"])?;
    let other_reply = play(&root, ["e2e4", "c7c5"])?;
    let single = (EVEN, single.as_slice());
    check_first_variation(&[single], Some(single))?;
    for changed in [
        (EVEN, other_root_move.as_slice()),
        (EVEN, other_reply.as_slice()),
        (Score::Centipawns(100), single.1),
    ] {
        assert_rejected(
            check_first_variation(&[changed], Some(single)),
            "first variation differs from the result at setting 1",
        )?;
    }
    Ok(())
}

#[test]
fn plain_minimax_passes_the_comparison_corpus() -> TestResult {
    check_comparison_corpus(|root, depth, requested| {
        Some(scored_variations(root, depth, requested))
    })
}

#[test]
#[ignore = "fisher-8hc.9 supplies the search that accepts a requested count"]
fn search_passes_the_comparison_corpus() -> TestResult {
    check_comparison_corpus(|root, depth, _requested| {
        crate::minimax_until_interrupted(root, depth, || false).map(|single| {
            if single.principal_variation.is_empty() {
                Vec::new()
            } else {
                vec![(single.score, single.principal_variation)]
            }
        })
    })
}

#[test]
fn corpus_rejects_a_duplicate_at_setting_three() -> TestResult {
    let result = check_comparison_corpus(|root, depth, requested| {
        let mut variations = scored_variations(root, depth, requested);
        if requested == 3
            && let [first, second, ..] = variations.as_mut_slice()
        {
            *second = first.clone();
        }
        Some(variations)
    });
    assert_rejected(
        result,
        "at depth 1, setting 3: variation 2: repeated root move",
    )
}

#[test]
fn corpus_rejects_a_different_first_variation_at_setting_255() -> TestResult {
    let result = check_comparison_corpus(|root, depth, requested| {
        let mut variations = scored_variations(root, depth, requested);
        if requested == 255
            && let [first, second, ..] = variations.as_mut_slice()
        {
            std::mem::swap(first, second);
        }
        Some(variations)
    });
    assert_rejected(
        result,
        "at depth 1, setting 255: first variation differs from the result at setting 1",
    )
}

#[test]
fn corpus_requires_a_finished_result_at_a_terminal_root() -> TestResult {
    let result = check_comparison_corpus(|root, depth, requested| {
        (!root.legal_moves().is_empty()).then(|| scored_variations(root, depth, requested))
    });
    assert_rejected(
        result,
        "k7/1Q6/2K5/8/8/8/8/8 b - - 0 1 at depth 1, setting 2: expected a finished result",
    )
}
