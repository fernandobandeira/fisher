#![cfg(test)]

use std::error::Error;

use rules::Position;

use super::plain_minimax::plain_minimax;

const POSITIONS: &str = include_str!("../../../../test-data/tied-root-moves.txt");

#[test]
fn the_previous_best_move_ties_with_an_earlier_legal_move() -> Result<(), Box<dyn Error>> {
    let entries: Vec<_> = POSITIONS
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    assert_eq!(entries.len(), 2, "expected the two D73 positions");
    for entry in entries {
        let (_, fen) = entry.split_once(' ').ok_or("expected setting and FEN")?;
        let position = Position::from_fen(fen)?;
        let previous = plain_minimax(&position, 1)
            .chosen_root_move()
            .ok_or("depth 1 has no best move")?;
        let next = plain_minimax(&position, 2);

        assert!(
            next.root_moves.contains(&(previous, next.score)),
            "{fen}: the depth-1 best move no longer has the best score at depth 2"
        );
        assert_ne!(
            next.chosen_root_move(),
            Some(previous),
            "{fen}: the depth-1 best move still comes first in legal order among the equal best"
        );
    }
    Ok(())
}

#[test]
fn the_tie_at_setting_two_crosses_the_last_requested_place() -> Result<(), Box<dyn Error>> {
    const REQUESTED: usize = 2;
    let fen = POSITIONS
        .lines()
        .find_map(|line| line.strip_prefix("2 "))
        .ok_or("missing position at setting 2")?;
    let position = Position::from_fen(fen)?;
    let previous = plain_minimax(&position, 1)
        .chosen_root_move()
        .ok_or("depth 1 has no best move")?;
    let next = plain_minimax(&position, 2);

    let equal_best: Vec<_> = next
        .root_moves
        .iter()
        .filter_map(|&(movement, score)| (score == next.score).then_some(movement))
        .collect();
    assert!(
        equal_best.len() > REQUESTED,
        "{fen}: only {} equal-best root moves at depth 2",
        equal_best.len()
    );
    let place = equal_best
        .iter()
        .position(|&movement| movement == previous)
        .ok_or("the depth-1 best move no longer has the best score at depth 2")?
        + 1;
    assert!(
        place > REQUESTED,
        "{fen}: the depth-1 best move is at place {place}, within setting {REQUESTED}"
    );
    Ok(())
}
