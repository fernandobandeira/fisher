#[path = "../src/tests/support/mate_in_two.rs"]
mod mate_in_two;
#[path = "../src/tests/support/perft.rs"]
mod perft;
#[path = "../src/tests/support/principal_variation.rs"]
mod principal_variation;

use std::{error::Error, num::NonZeroU8};

use perft::check_perft;
use rules::Position;
use search::{FinishedResult, Score, SearchResult, minimax, minimax_variations_until_interrupted};
use shakmaty::{CastlingMode, Chess, Position as _, fen::Fen};

type TestResult = Result<(), Box<dyn Error>>;

fn search_nodes(fen: &str, depth: u8) -> Result<u64, Box<dyn Error>> {
    Ok(minimax(&Position::from_fen(fen)?, depth).nodes)
}

#[test]
fn depth_one_counts_each_legal_move_from_the_published_positions() -> TestResult {
    check_perft(1, search_nodes)
}

#[test]
fn starting_position_depth_two_counts_only_moves_played() {
    // The first root move costs 1 + 20 nodes, the other 19 cost 1 + 1 each.
    assert_eq!(minimax(&Position::starting(), 2).nodes, 59);
}

fn reference_position(fen: &str) -> Result<Chess, Box<dyn Error>> {
    Ok(fen.parse::<Fen>()?.into_position(CastlingMode::Standard)?)
}

/// Searches `fen` and checks the principal variation under D5.
fn search_and_replay(fen: &str, depth: u8) -> Result<SearchResult, Box<dyn Error>> {
    let result = minimax(&Position::from_fen(fen)?, depth);
    assert_eq!(
        result.principal_variation.first().copied(),
        result.best_move,
        "{fen}"
    );
    let score = match result.score {
        Score::Centipawns(value) => value,
        Score::Mate { plies, winning } => {
            let value = principal_variation::MATE_SCORE - i32::from(plies);
            if winning { value } else { -value }
        }
    };
    principal_variation::verify(
        &reference_position(fen)?,
        depth,
        score,
        result.principal_variation.iter().copied().map(move_text),
    )?;
    Ok(result)
}

#[test]
fn replay_rejects_a_variation_that_does_not_support_the_score() -> TestResult {
    let start = reference_position("7k/q7/8/8/8/8/8/R6K w - - 0 1")?;
    // After Rxa7 White is a rook ahead. A variation that keeps the queen on
    // the board, or stops short of the depth, does not support that score.
    principal_variation::verify(&start, 1, 500, ["a1a7"])?;
    assert!(principal_variation::verify(&start, 1, 500, ["a1a2"]).is_err());
    assert!(principal_variation::verify(&start, 1, -400, ["a1a7"]).is_err());
    assert!(principal_variation::verify(&start, 2, 500, ["a1a7"]).is_err());
    assert!(principal_variation::verify(&start, 1, 500, ["a1a8"]).is_err());
    Ok(())
}

fn check_mate_in_two(records: impl IntoIterator<Item = mate_in_two::Record>) -> TestResult {
    for record in records {
        let result = search_and_replay(record.fen, 4)?;
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
            result.best_move.map(move_text).as_deref(),
            Some(record.published_move),
            "{}",
            record.id
        );
    }
    Ok(())
}

#[test]
fn first_ten_published_mates_in_two() -> TestResult {
    // The first record is Lichess puzzle 000Zo. Its README says exactly one
    // legal move forces the checkmate, so the chosen move is the published one.
    check_mate_in_two(mate_in_two::records()?.into_iter().take(10))
}

#[test]
#[ignore = "run in release mode for every published mate-in-two position"]
fn all_published_mates_in_two() -> TestResult {
    let records = mate_in_two::records()?;
    // The published table is documented as containing 100 positions.
    assert_eq!(records.len(), 100);
    check_mate_in_two(records)
}

#[test]
fn mate_in_one_scores_as_a_win() -> TestResult {
    // Qb7 gives check and covers a7 and b8. The white king protects b7.
    let result = search_and_replay("k7/8/1QK5/8/8/8/8/8 w - - 0 1", 2)?;
    assert_eq!(
        result.score,
        Score::Mate {
            plies: 1,
            winning: true
        }
    );
    assert_eq!(result.best_move.map(move_text).as_deref(), Some("b6b7"));
    Ok(())
}

#[test]
fn stalemate_is_avoided_before_the_horizon_and_scored_as_material_on_it() -> TestResult {
    let fen = "k7/8/1p6/1Q6/8/8/8/7K w - - 0 1";
    // Qxb6 leaves a8 unchecked, with a7, b7 and b8 covered. At depth 2,
    // stalemate scores zero, so White prefers retaining its 800-centipawn lead.
    assert_eq!(search_and_replay(fen, 2)?.score, Score::Centipawns(800));
    // At depth 1 the stalemate stands on the horizon, so Qxb6 is scored as
    // material, 900, and every other move keeps 800.
    let result = search_and_replay(fen, 1)?;
    assert_eq!(result.score, Score::Centipawns(900));
    assert_eq!(result.best_move.map(move_text).as_deref(), Some("b5b6"));
    let mut position = reference_position(fen)?;
    let capture = position
        .legal_moves()
        .into_iter()
        .find(|movement| movement.is_capture())
        .ok_or("missing Qxb6")?;
    position.play_unchecked(capture);
    assert!(position.is_stalemate());
    Ok(())
}

#[test]
fn free_capture_scores_the_material_left() -> TestResult {
    // Rxa7 wins the queen, leaving White a rook ahead.
    let result = search_and_replay("7k/q7/8/8/8/8/8/R6K w - - 0 1", 1)?;
    assert_eq!(result.score, Score::Centipawns(500));
    Ok(())
}

#[test]
fn free_knight_capture_keeps_the_remaining_material_after_a_reply() -> TestResult {
    // Rxa7 wins the knight. Black has legal replies but cannot capture any
    // white piece. The rooks cancel, leaving 300 + 300 + 900 centipawns.
    let result = search_and_replay("7k/n7/6r1/8/8/8/8/RNBQ3K w - - 0 1", 2)?;
    assert_eq!(result.score, Score::Centipawns(1_500));
    Ok(())
}

#[test]
fn quiet_move_beats_a_capture_that_allows_a_recapture() -> TestResult {
    let fen = "7k/8/2p5/3p4/8/8/8/3Q3K w - - 0 1";
    // White starts 900 - 200 = 700 centipawns ahead. Qxd5 is the only capture.
    // At depth 1 it scores the material left, 900 - 100 = 800. At depth 2,
    // cxd5 answers it and leaves Black a pawn ahead, -100. A quiet move such as
    // Qd2 keeps the 700 lead: no black piece attacks the queen there, and no
    // white move checkmates or stalemates Black.
    assert_eq!(search_and_replay(fen, 1)?.score, Score::Centipawns(800));
    assert_eq!(search_and_replay(fen, 2)?.score, Score::Centipawns(700));
    Ok(())
}

#[test]
fn checkmated_root_has_no_move_variation_or_nodes() -> TestResult {
    // Qb7 and Qg7 give check, protected by the white king, and cover every
    // square the black king could go to.
    for fen in [
        "k7/1Q6/2K5/8/8/8/8/8 b - - 0 1",
        "7k/6Q1/6K1/8/8/8/8/8 b - - 0 1",
    ] {
        for depth in [1, 4] {
            let result = search_and_replay(fen, depth)?;
            assert_eq!(result.best_move, None, "{fen}");
            assert_eq!(
                result.score,
                Score::Mate {
                    plies: 0,
                    winning: false
                },
                "{fen}"
            );
            assert_eq!(result.nodes, 0, "{fen}");
            assert_eq!(result.principal_variation, [], "{fen}");
        }
    }
    Ok(())
}

#[test]
fn stalemated_root_has_no_move_variation_or_nodes() -> TestResult {
    // The black king on h8 is not in check, and the queen on f7 and the king
    // on g6 cover g8, g7 and h7.
    let result = search_and_replay("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1", 4)?;
    assert_eq!(result.best_move, None);
    assert_eq!(result.score, Score::Centipawns(0));
    assert_eq!(result.nodes, 0);
    assert_eq!(result.principal_variation, []);
    Ok(())
}

#[test]
fn depth_zero_has_no_move_or_variation() -> TestResult {
    let result = search_and_replay(&Position::starting().to_fen(), 0)?;
    assert_eq!(result.best_move, None);
    assert_eq!(result.score, Score::Centipawns(0));
    assert_eq!(result.nodes, 0);
    assert_eq!(result.principal_variation, []);
    Ok(())
}

#[test]
fn equal_scores_keep_the_first_legal_move() {
    let position = Position::starting();
    // None of the starting position's 20 legal moves changes the material.
    let first = position.legal_moves().into_iter().next();
    let result = minimax(&position, 1);
    assert_eq!(result.best_move, first);
    assert_eq!(result.score, Score::Centipawns(0));
    assert_eq!(result.nodes, 20);
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
fn checks_for_an_interrupt_before_each_node() {
    let position = Position::starting();
    // Starting-position count for a single depth of 2, from fisher-4yn.
    let nodes = 59;
    for interrupt_at in [1, 2, 22, nodes] {
        let mut interrupt_checks = 0;
        let result = search_variations(&position, 2, NonZeroU8::MIN, || {
            interrupt_checks += 1;
            interrupt_checks >= interrupt_at
        });
        assert!(result.is_none(), "interrupt at node {interrupt_at}");
        assert_eq!(interrupt_checks, interrupt_at);
    }
    let mut interrupt_checks = 0;
    let result = search_variations(&position, 2, NonZeroU8::MIN, || {
        interrupt_checks += 1;
        false
    });
    assert_eq!(interrupt_checks, nodes);
    assert_eq!(result.map(SearchResult::from), Some(minimax(&position, 2)));
}

#[test]
fn does_not_check_for_an_interrupt_at_the_root() {
    let position = Position::starting();
    assert_eq!(
        search_variations(&position, 0, NonZeroU8::MIN, || true).map(SearchResult::from),
        Some(minimax(&position, 0))
    );
}

fn search_variations(
    root: &Position,
    depth: u8,
    requested: NonZeroU8,
    mut interrupted: impl FnMut() -> bool,
) -> Option<FinishedResult> {
    // Exercise all outcomes of one search instantiation for LLVM region coverage.
    let interrupted: &mut dyn FnMut() -> bool = &mut interrupted;
    minimax_variations_until_interrupted(root, depth, requested, interrupted)
}

#[test]
fn several_variations_keep_legal_move_order_on_ties() -> TestResult {
    let root = Position::starting();
    let legal_moves = root.legal_moves();
    for requested in [2, 3, 255] {
        let requested = NonZeroU8::new(requested).ok_or("expected a positive setting")?;
        // No move changes the material in the first two plies from the
        // starting position. D48 keeps legal move order on these ties.
        let expected: Vec<_> = legal_moves
            .iter()
            .take(usize::from(requested.get()))
            .map(Some)
            .collect();
        for depth in [1, 2] {
            let result = search_variations(&root, depth, requested, || false)
                .ok_or("expected a finished result")?;
            let actual: Vec<_> = result
                .variations
                .iter()
                .map(|(_, moves)| moves.first())
                .collect();
            assert_eq!(actual, expected);
        }
    }
    Ok(())
}

#[test]
fn tied_root_moves_keep_legal_order_across_the_last_requested_place() -> TestResult {
    let root = Position::from_fen("7k/q7/8/8/8/8/8/R6K w - - 0 1")?;
    // At depth one, Rxa7 is worth 500 cp. Every other move leaves the
    // rook and queen on the board, worth -400 cp for White.
    let (captures, quiet): (Vec<_>, Vec<_>) = root
        .legal_moves()
        .into_iter()
        .partition(|&movement| move_text(movement) == "a1a7");
    assert_eq!(captures.len(), 1);
    assert!(quiet.len() > 2, "the tie must extend past rank three");
    let expected: Vec<_> = captures
        .into_iter()
        .map(|movement| (Score::Centipawns(500), vec![movement]))
        .chain(
            quiet
                .into_iter()
                .take(2)
                .map(|movement| (Score::Centipawns(-400), vec![movement])),
        )
        .collect();
    let requested = NonZeroU8::new(3).ok_or("expected a positive setting")?;
    let result =
        search_variations(&root, 1, requested, || false).ok_or("expected a finished result")?;
    assert_eq!(result.variations, expected);
    Ok(())
}

#[test]
fn one_requested_variation_matches_the_single_search() -> TestResult {
    let root = Position::starting();
    let result = search_variations(&root, 2, NonZeroU8::MIN, || false)
        .ok_or("expected a finished result")?;
    assert_eq!(result.variations.len(), 1);
    assert_eq!(SearchResult::from(result), minimax(&root, 2));
    Ok(())
}

#[test]
fn refuses_a_partial_set_of_variations() -> TestResult {
    let root = Position::starting();
    for requested in [1, 2, 3, 255] {
        let requested = NonZeroU8::new(requested).ok_or("expected a positive setting")?;
        let mut checks = 0;
        let finished = search_variations(&root, 2, requested, || {
            checks += 1;
            false
        })
        .ok_or("expected a finished result")?;
        assert_eq!(finished.nodes, checks);
        for interrupt_at in [1, finished.nodes] {
            let mut checks = 0;
            let result = search_variations(&root, 2, requested, || {
                checks += 1;
                checks >= interrupt_at
            });
            assert!(result.is_none(), "setting {requested}, node {interrupt_at}");
            assert_eq!(checks, interrupt_at);
        }
    }
    Ok(())
}

#[test]
fn a_finished_root_can_have_no_variations() -> TestResult {
    let starting = Position::starting();
    let checkmate = Position::from_fen("k7/1Q6/2K5/8/8/8/8/8 b - - 0 1")?;
    let stalemate = Position::from_fen("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1")?;
    for (root, depth) in [(starting, 0), (checkmate, 1), (stalemate, 1)] {
        for requested in [1, 2, 3, 255] {
            let requested = NonZeroU8::new(requested).ok_or("expected a positive setting")?;
            let result = search_variations(&root, depth, requested, || true)
                .ok_or("expected a finished result")?;
            assert_eq!(result.variations, []);
            assert_eq!(SearchResult::from(result), minimax(&root, depth));
        }
    }
    Ok(())
}
