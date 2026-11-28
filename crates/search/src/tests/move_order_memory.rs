#![cfg(test)]

use std::{cmp::Reverse, error::Error, num::NonZeroU8};

use rules::Position;

use super::{
    comparison::{check_each_prefix, move_text, score_value, visit_comparison_corpus},
    plain_minimax::plain_minimax,
    several_variations::check_variations,
};
use crate::{MoveOrderMemory, SearchResult, minimax, minimax_variations_until_interrupted};

type TestResult = Result<(), Box<dyn Error>>;

#[test]
fn a_previous_best_loses_priority_when_it_is_no_longer_equal_best() -> TestResult {
    // Astra's PR #131 review position. At depth 2, f8h6 overtakes f8a3,
    // and f8e7 precedes f8a3 in legal order at their tied score.
    let root = Position::from_fen("4kb2/8/7B/8/8/N7/8/R3K3 b - - 0 10")?;
    check_carried_rankings(&root, 2, [NonZeroU8::new(2).ok_or("expected a setting")?])
}

#[test]
fn carried_memory_keeps_the_full_minimax_ranking() -> TestResult {
    let settings = [2, 3, 255].map(|setting| NonZeroU8::new(setting).expect("positive setting"));
    visit_comparison_corpus(|root, depth| check_carried_rankings(root, depth, settings))
}

fn check_carried_rankings<const COUNT: usize>(
    root: &Position,
    last_depth: u8,
    settings: [NonZeroU8; COUNT],
) -> TestResult {
    let mut memories = settings.map(|_| MoveOrderMemory::default());
    let mut previous_best = None;
    for depth in 1..=last_depth {
        let plain = plain_minimax(root, depth);
        let mut ranked = plain.root_moves;
        // The stable sort preserves plain minimax's legal order within a tie.
        ranked.sort_by_key(|&(movement, score)| {
            (
                Reverse(score_value(score)),
                score != plain.score || Some(movement) != previous_best,
            )
        });
        previous_best = ranked.first().map(|&(movement, _)| movement);
        for (setting, memory) in settings.into_iter().zip(&mut memories) {
            let result =
                minimax_variations_until_interrupted(root, depth, setting, memory, || false)
                    .ok_or("search did not finish")?;
            let case = || format!("{} at depth {depth}, setting {setting}", root.to_fen());
            assert_eq!(result.score, plain.score, "{}", case());
            let actual = result.variations.iter().map(|(score, moves)| {
                moves
                    .first()
                    .map(|&movement| (move_text(movement), *score))
                    .ok_or("missing root move")
            });
            let expected = ranked
                .iter()
                .take(usize::from(setting.get()))
                .map(|&(movement, score)| (move_text(movement), score))
                .collect::<Vec<_>>();
            assert_eq!(
                actual.collect::<Result<Vec<_>, _>>()?,
                expected,
                "{}",
                case()
            );
        }
    }
    Ok(())
}

#[test]
fn carried_memory_keeps_the_minimax_score_and_an_equal_best_move() -> Result<(), Box<dyn Error>> {
    visit_comparison_corpus(|position, depth| {
        let mut memory = MoveOrderMemory::default();
        let search = |depth, memory: &mut MoveOrderMemory| {
            minimax_variations_until_interrupted(position, depth, NonZeroU8::MIN, memory, || false)
                .ok_or("search did not finish")
        };
        for earlier in 1..depth {
            search(earlier, &mut memory)?;
        }
        let actual = SearchResult::from(search(depth, &mut memory)?);
        let expected = plain_minimax(position, depth);
        let case = || format!("{} at depth {depth}", position.to_fen());
        assert_eq!(actual.score, expected.score, "{}", case());
        match actual.best_move {
            Some(movement) => assert!(
                expected.root_moves.contains(&(movement, expected.score)),
                "the best move was not equal best at {}",
                case()
            ),
            None => assert!(expected.root_moves.is_empty(), "{}", case()),
        }
        check_each_prefix(position, depth, actual.score, &actual.principal_variation)?;
        Ok(())
    })
}

#[test]
fn a_remembered_move_without_a_legal_match_leaves_the_result_unchanged()
-> Result<(), Box<dyn Error>> {
    let root = Position::starting();
    let mut memory = MoveOrderMemory::default();
    let previous =
        minimax_variations_until_interrupted(&root, 1, NonZeroU8::MIN, &mut memory, || false)
            .map(SearchResult::from)
            .ok_or("depth 1 did not finish")?;
    let next = previous.best_move.ok_or("missing root move")?.play();
    // The other color is now to move, so it cannot play the remembered move.
    let expected = minimax(&next, 2);
    let actual =
        minimax_variations_until_interrupted(&next, 2, NonZeroU8::MIN, &mut memory, || false)
            .map(SearchResult::from);
    assert_eq!(actual, Some(expected));
    Ok(())
}

#[test]
fn ties_keep_the_previous_best_then_follow_legal_order() -> Result<(), Box<dyn Error>> {
    // The plain minimax checks in tied_root_moves.rs establish the D73 ties.
    let positions = include_str!("../../../../test-data/tied-root-moves.txt");
    for entry in positions
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
        let (setting, fen) = entry.split_once(' ').ok_or("expected setting and FEN")?;
        let root = Position::from_fen(fen)?;
        let setting = setting.parse::<NonZeroU8>()?;
        let previous = plain_minimax(&root, 1)
            .chosen_root_move()
            .ok_or("depth 1 has no best move")?;
        let expected = plain_minimax(&root, 2);
        let tied = std::iter::once(previous).chain(expected.root_moves.iter().filter_map(
            |&(movement, score)| {
                (score == expected.score && movement != previous).then_some(movement)
            },
        ));
        let mut memory = MoveOrderMemory::default();
        minimax_variations_until_interrupted(&root, 1, setting, &mut memory, || false)
            .ok_or("depth 1 did not finish")?;
        let result = minimax_variations_until_interrupted(&root, 2, setting, &mut memory, || false)
            .ok_or("depth 2 did not finish")?;
        let variations: Vec<_> = result
            .variations
            .iter()
            .map(|(score, moves)| (*score, moves.as_slice()))
            .collect();
        check_variations(&root, 2, setting.get(), &variations)?;
        let actual: Vec<_> = variations
            .iter()
            .take_while(|(score, _)| *score == expected.score)
            .filter_map(|(_, moves)| moves.first().copied())
            .collect();
        assert_eq!(
            actual,
            tied.take(usize::from(setting.get())).collect::<Vec<_>>(),
            "{fen}"
        );
    }
    Ok(())
}
