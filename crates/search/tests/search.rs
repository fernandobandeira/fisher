#[path = "support/mate_in_two.rs"]
mod mate_in_two;
#[path = "support/principal_variation.rs"]
mod principal_variation;
#[path = "support/reference_minimax.rs"]
mod reference;
mod support;

use std::error::Error;

use rules::Position;
use search::{Score, SearchResult, minimax};
use shakmaty::{Chess, EnPassantMode, Position as _, fen::Fen};
use support::{check_perft, reference_position};

fn search_nodes(fen: &str, depth: u8) -> Result<u64, Box<dyn Error>> {
    Ok(minimax(&Position::from_fen(fen)?, depth).nodes)
}

#[test]
fn node_count_matches_published_perft_at_two_plies() -> Result<(), Box<dyn Error>> {
    check_perft(2, search_nodes)
}

#[test]
#[ignore = "run in release mode for the published four-ply counts"]
fn node_count_matches_published_perft_at_four_plies() -> Result<(), Box<dyn Error>> {
    check_perft(4, search_nodes)
}

fn check_mate_in_two(
    records: impl IntoIterator<Item = mate_in_two::Record>,
) -> Result<(), Box<dyn Error>> {
    for record in records {
        let position = reference_position(record.fen)?;
        let result = assert_matches_reference(&position, 4)?;
        assert_eq!(
            result.score,
            Score::Mate {
                plies: 3,
                winning: true
            },
            "{}",
            record.id
        );
        assert_eq!(
            result.chosen_move.map(move_text).as_deref(),
            Some(record.published_move),
            "{}",
            record.id
        );
    }
    Ok(())
}

#[test]
fn first_ten_published_mates_in_two() -> Result<(), Box<dyn Error>> {
    check_mate_in_two(mate_in_two::records()?.into_iter().take(10))
}

#[test]
#[ignore = "run in release mode for every published mate-in-two position"]
fn all_published_mates_in_two() -> Result<(), Box<dyn Error>> {
    let records = mate_in_two::records()?;
    // The published table is documented as containing 100 positions.
    assert_eq!(records.len(), 100);
    check_mate_in_two(records)
}

#[test]
fn mate_on_the_horizon_scores_as_material() -> Result<(), Box<dyn Error>> {
    for record in mate_in_two::records()?.into_iter().take(10) {
        let position = reference_position(record.fen)?;
        assert_matches_reference(&position, 3)?;
    }
    Ok(())
}

#[test]
fn stalemate_on_the_horizon_scores_as_material() -> Result<(), Box<dyn Error>> {
    let fen = "k7/8/1p6/1Q6/8/8/8/7K w - - 0 1";
    let position: Chess = reference_position(fen)?;
    let chosen = reference::minimax(&position, 1).chosen_move;
    let chosen = chosen.ok_or("reference found no legal move")?;
    assert!(position.clone().play(chosen)?.is_stalemate());
    assert_matches_reference(&position, 1).map(drop)
}

#[test]
fn equal_scores_keep_the_first_legal_move() {
    let position = Position::starting();
    // None of the starting position's 20 legal moves changes the material.
    let first = position.legal_moves().into_iter().next();
    let result = minimax(&position, 1);
    assert_eq!(result.chosen_move, first);
    assert_eq!(result.score, Score::Centipawns(0));
    assert_eq!(result.nodes, 20);
}

#[test]
fn score_and_nodes_match_reference_on_random_reachable_positions() -> Result<(), Box<dyn Error>> {
    for mut seed in 1_u64..=3 {
        let mut position = Chess::default();
        for _ in 0..12 {
            let moves = position.legal_moves();
            if moves.is_empty() {
                break;
            }
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            let index = usize::try_from((seed >> 32) % u64::try_from(moves.len())?)?;
            let chosen = moves.get(index).copied().ok_or("missing reference move")?;
            position = position.play(chosen)?;

            assert_matches_reference(&position, 2)?;
        }
    }
    Ok(())
}

#[test]
fn terminal_positions_and_depth_zero_have_empty_variations() -> Result<(), Box<dyn Error>> {
    for fen in [
        "7k/6Q1/6K1/8/8/8/8/8 b - - 0 1",
        "7k/5Q2/6K1/8/8/8/8/8 b - - 0 1",
    ] {
        let position = reference_position(fen)?;
        assert!(position.is_checkmate() || position.is_stalemate(), "{fen}");
        let result = assert_matches_reference(&position, 4)?;
        assert_eq!(result.principal_variation, [], "{fen}");
    }
    let result = assert_matches_reference(&Chess::default(), 0)?;
    assert_eq!(result.principal_variation, []);
    Ok(())
}

fn assert_matches_reference(position: &Chess, depth: u8) -> Result<SearchResult, Box<dyn Error>> {
    let expected = reference::minimax(position, depth);
    let fen = Fen::from_position(position, EnPassantMode::Always).to_string();
    let result = minimax(&Position::from_fen(&fen)?, depth);
    let score = match result.score {
        Score::Centipawns(value) => value,
        Score::Mate { plies, winning } => {
            let value = reference::MATE_SCORE - i32::from(plies);
            if winning { value } else { -value }
        }
    };
    assert_eq!(score, expected.score, "{fen}");
    assert_eq!(result.nodes, expected.nodes, "{fen}");
    assert_eq!(
        result.principal_variation.first().copied(),
        result.chosen_move,
        "{fen}"
    );

    principal_variation::verify(
        position,
        depth,
        expected.score,
        result.principal_variation.iter().copied().map(move_text),
    )?;
    Ok(result)
}

fn move_text(movement: rules::Move) -> String {
    let promotion = match movement.promotion() {
        Some(rules::Promotion::Knight) => "n",
        Some(rules::Promotion::Bishop) => "b",
        Some(rules::Promotion::Rook) => "r",
        Some(rules::Promotion::Queen) => "q",
        None => "",
    };
    format!("{}{}{promotion}", movement.from(), movement.to())
}

#[test]
fn permission_is_required_before_each_node() {
    use search::minimax_with_permission;

    let position = Position::starting();
    // Published starting-position perft counts.
    let nodes = 20 + 400;
    for refusal in [1, 2, 22, nodes] {
        let mut permission_checks = 0;
        let result = minimax_with_permission(&position, 2, &mut || {
            permission_checks += 1;
            permission_checks < refusal
        });
        assert!(result.is_none(), "refusal at node {refusal}");
        assert_eq!(permission_checks, refusal);
    }
    let mut permission_checks = 0;
    let result = minimax_with_permission(&position, 2, &mut || {
        permission_checks += 1;
        true
    });
    assert_eq!(permission_checks, nodes);
    assert_eq!(result, Some(minimax(&position, 2)));
}

#[test]
fn permission_is_unused_if_only_the_root_is_searched() {
    use search::minimax_with_permission;

    let position = Position::starting();
    assert_eq!(
        minimax_with_permission(&position, 0, &mut || false),
        Some(minimax(&position, 0))
    );
}
