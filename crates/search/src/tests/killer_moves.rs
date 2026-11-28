#![cfg(test)]

use std::{convert::Infallible, num::NonZeroU8};

use rules::{Color, Move, Position};

use crate::history::History;
use crate::move_order::{KillerMoves, MovePriority, capture_priority, ordered_moves};
use crate::tests::comparison::move_text;
use crate::{
    BestReply, MoveOrderMemory, SearchProgress, minimax_variations_until_interrupted, search,
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
    let mut history = History::default();
    history.remember(Color::White, &quiet_rook, 1);
    history.remember(Color::White, &knight, 1);

    let priorities = |previous| {
        ordered_moves(&position, &moves, previous, killers, &history)
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
    assert!(without_previous.get("a7a8b") == Some(&MovePriority::History(82)));

    let previous = [knight];
    assert!(priorities(&previous).get("a7a8n") == Some(&MovePriority::Remembered));
    assert_eq!(
        ordered_moves(&position, &moves, &previous, killers, &history).count(),
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
    let mut history = History::default();
    let mut progress = SearchProgress {
        nodes: 0,
        killers: &mut killers,
        history: &mut history,
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
    assert_eq!(
        history.destinations(Color::Black, reply.from()),
        &[(reply.to(), 82)]
    );
    assert_eq!(history.destinations(Color::White, reply.from()), []);
}

#[test]
fn captures_do_not_fill_killer_slots() {
    let position = Position::from_fen("7k/8/8/8/8/8/n7/R6K w - - 0 1").unwrap();
    let moves = position.legal_moves();
    let capture = find_move(&moves, "a1a2");
    let mut killers = KillerMoves::default();
    let mut history = History::default();

    SearchProgress {
        nodes: 0,
        killers: std::slice::from_mut(&mut killers),
        history: &mut history,
    }
    .remember_cutoff(
        &position,
        0,
        &capture,
        capture_priority(&position, &capture),
        1,
    );
    assert!(killers.priority(&capture) == MovePriority::Quiet);
    assert_eq!(history.destinations(Color::White, capture.from()), []);
    assert!(MovePriority::LatestKiller.is_quiet());
    assert!(!MovePriority::QueenPromotion.is_quiet());
    assert!(MovePriority::LatestKiller.is_killer());
    assert!(MovePriority::OlderKiller.is_killer());
    assert!(!MovePriority::Quiet.is_killer());
}

#[test]
fn history_priorities_follow_the_depth_bonus_and_butterfly_key() {
    let root = Position::starting();
    let moves = root.legal_moves();
    let first = find_move(&moves, "a2a3");
    let same_origin = find_move(&moves, "a2a4");
    let other_origin = find_move(&moves, "b2b3");
    let mut history = History::default();
    history.remember(Color::White, &first, 10);
    history.remember(Color::White, &first, 11);
    history.remember(Color::White, &same_origin, 2);
    history.remember(Color::White, &other_origin, 4);

    let priorities: std::collections::BTreeMap<_, _> =
        ordered_moves(&root, &moves, &[], KillerMoves::default(), &history)
            .map(|(_, movement, priority, _)| (move_text(movement), priority))
            .collect();
    // Stockfish 17 gives each deep cutoff 1596 before damping. The
    // second adds 1596 - floor(1596 * 1596 / 7183).
    assert!(priorities.get("a2a3") == Some(&MovePriority::History(2838)));
    assert!(priorities.get("a2a4") == Some(&MovePriority::History(272)));
    assert!(priorities.get("b2b3") == Some(&MovePriority::History(652)));
    assert!(priorities.get("b2b4") == Some(&MovePriority::Quiet));
}

#[test]
fn an_earlier_quiet_move_can_tie_a_killer_only_above_alpha() {
    let best = BestReply {
        score: 100,
        prioritized_quiet: Some(2),
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
