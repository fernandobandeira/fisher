#![cfg(test)]

use std::{convert::Infallible, num::NonZeroU8};

use rules::{Move, Position};

use crate::move_order::{KillerMoves, MovePriority, capture_priority, ordered_moves};
use crate::tests::comparison::move_text;
use crate::{
    BestReply, MoveOrderMemory, SearchProgress, minimax_variations_until_interrupted,
    remember_cutoff, search,
};

#[test]
fn the_latest_distinct_cutoff_replaces_the_older_killer() {
    let moves = Position::starting().legal_moves();
    let find = |text| find_move(&moves, text);
    let first = find("a2a3");
    let second = find("b2b3");
    let third = find("c2c3");

    let mut killers = KillerMoves::default();
    killers.remember(&first);
    killers.remember(&second);
    killers.remember(&second);
    assert!(killers.priority(&first) == MovePriority::OlderKiller);
    assert!(killers.priority(&second) == MovePriority::LatestKiller);

    killers.remember(&first);
    assert!(killers.priority(&first) == MovePriority::LatestKiller);
    assert!(killers.priority(&second) == MovePriority::OlderKiller);

    killers.remember(&third);
    assert!(killers.priority(&first) == MovePriority::OlderKiller);
    assert!(killers.priority(&second) == MovePriority::Quiet);
    assert!(killers.priority(&third) == MovePriority::LatestKiller);
}

#[test]
fn remembered_variation_and_captures_take_priority_over_killer_slots() {
    let quiet_position = Position::from_fen("7k/8/8/8/8/8/8/R6K w - - 0 1").unwrap();
    let position = Position::from_fen("7k/P7/8/8/8/8/n7/R6K w - - 0 1").unwrap();
    let quiet_moves = quiet_position.legal_moves();
    let moves = position.legal_moves();
    let quiet_rook = find_move(&quiet_moves, "a1a2");
    let knight = find_move(&moves, "a7a8n");
    let mut killers = KillerMoves::default();
    killers.remember(&quiet_rook);
    killers.remember(&knight);

    let priorities = |previous| {
        ordered_moves(&position, &moves, previous, killers)
            .map(|(_, movement, priority, _)| (move_text(movement), priority))
            .collect::<std::collections::BTreeMap<_, _>>()
    };
    let without_previous = priorities(&[]);
    assert!(matches!(
        without_previous.get("a1a2"),
        Some(MovePriority::Capture { .. })
    ));
    assert!(without_previous.get("a7a8q") == Some(&MovePriority::QueenPromotion));
    assert!(without_previous.get("a7a8n") == Some(&MovePriority::LatestKiller));
    assert!(without_previous.get("a7a8b") == Some(&MovePriority::Quiet));

    let previous = [knight];
    assert!(priorities(&previous).get("a7a8n") == Some(&MovePriority::Remembered));
    assert_eq!(
        ordered_moves(&position, &moves, &previous, killers).count(),
        moves.len()
    );
}

#[test]
fn a_quiet_cutoff_fills_only_the_killer_slot_at_its_ply() {
    // The rook checks along rank 8 and the white king covers g7.
    // Kh7 is the only legal reply and leaves Black 500 centipawns down.
    let position = Position::from_fen("R6k/8/5K2/8/8/8/8/8 b - - 0 1").unwrap();
    let moves = position.legal_moves();
    let reply = find_move(&moves, "h8h7");
    let mut killers = [KillerMoves::default(); 3];
    let mut progress = SearchProgress {
        nodes: 0,
        killers: &mut killers,
    };
    let Ok(score) = search(
        &position,
        1,
        (-501, -500),
        &mut progress,
        &mut [Vec::new()],
        &[],
        &mut || Ok::<(), Infallible>(()),
    );

    assert_eq!(score, -500);
    assert!(killers[0].priority(&reply) == MovePriority::Quiet);
    assert!(killers[1].priority(&reply) == MovePriority::LatestKiller);
    assert!(killers[2].priority(&reply) == MovePriority::Quiet);
}

#[test]
fn captures_do_not_fill_killer_slots() {
    let position = Position::from_fen("7k/8/8/8/8/8/n7/R6K w - - 0 1").unwrap();
    let moves = position.legal_moves();
    let capture = find_move(&moves, "a1a2");
    let mut killers = KillerMoves::default();

    remember_cutoff(
        Some(&mut killers),
        &capture,
        capture_priority(&position, &capture),
    );
    assert!(killers.priority(&capture) == MovePriority::Quiet);
    assert!(MovePriority::LatestKiller.is_quiet());
    assert!(!MovePriority::QueenPromotion.is_quiet());
    assert!(MovePriority::LatestKiller.is_killer());
    assert!(MovePriority::OlderKiller.is_killer());
    assert!(!MovePriority::Quiet.is_killer());
}

#[test]
fn an_earlier_quiet_move_can_tie_a_killer_only_above_alpha() {
    let best = BestReply {
        score: 100,
        killer_index: Some(2),
    };

    assert!(best.wins_tie(99, 1, MovePriority::Quiet));
    assert!(!best.wins_tie(100, 1, MovePriority::Quiet));
    assert!(!best.wins_tie(99, 2, MovePriority::Quiet));
    assert!(!best.wins_tie(99, 1, MovePriority::QueenPromotion));
}

#[test]
fn an_interrupted_reply_keeps_the_last_finished_principal_variation() {
    let position = Position::starting();
    let mut memory = MoveOrderMemory::default();
    assert!(
        minimax_variations_until_interrupted(&position, 1, NonZeroU8::MIN, &mut memory, || false)
            .is_some()
    );
    let previous = memory.principal_variation.clone();
    let mut checks = 0;
    let result =
        minimax_variations_until_interrupted(&position, 2, NonZeroU8::MIN, &mut memory, || {
            checks += 1;
            checks == 2
        });

    assert!(result.is_none());
    assert_eq!(checks, 2);
    assert_eq!(memory.principal_variation, previous);
}

fn find_move(moves: &[Move], text: &str) -> Move {
    moves
        .iter()
        .copied()
        .find(|&movement| move_text(movement) == text)
        .unwrap()
}
