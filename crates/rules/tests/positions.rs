use std::{io, ops::RangeInclusive, time::Instant};

use rules::{Move, Position, Promotion, Square};
use shakmaty::{CastlingMode, Chess, EnPassantMode, Position as _, fen::Fen, uci::UciMove};

const PERFT: &str = include_str!("../../../expectations/perft.epd");

fn perft_positions() -> impl Iterator<Item = (&'static str, &'static str)> {
    PERFT.lines().filter_map(|line| {
        if line.trim().starts_with('#') {
            return None;
        }
        line.split_once(';')
            .map(|(fen, depths)| (fen.trim(), depths))
    })
}

fn published_perft_count(
    fen: &str,
    counts: &str,
    depth: u8,
) -> Result<u128, Box<dyn std::error::Error>> {
    Ok(counts
        .split(';')
        .filter_map(|count| count.trim().split_once(' '))
        .find_map(|(label, count)| {
            (label.strip_prefix('D')?.parse::<u8>().ok()? == depth).then_some(count)
        })
        .ok_or_else(|| format!("no published perft at depth {depth} for {fen}"))?
        .parse::<u128>()?)
}

fn reference_games(
    start: &Chess,
    seeds: RangeInclusive<u64>,
    mut visit: impl FnMut(&Chess, Option<shakmaty::Move>),
) -> Result<(), Box<dyn std::error::Error>> {
    for mut seed in seeds {
        let mut position = start.clone();
        for _ in 0..128 {
            let moves = position.legal_moves();
            if moves.is_empty() {
                visit(&position, None);
                break;
            }
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            let count = u64::try_from(moves.len())?;
            let index = usize::try_from((seed >> 32) % count)?;
            let chosen = moves.get(index).copied().ok_or("missing reference move")?;
            visit(&position, Some(chosen));
            position = position.play(chosen)?;
        }
    }
    Ok(())
}

#[test]
fn starting_position_matches_reference_fen() {
    let initial = Fen::from_position(&Chess::default(), EnPassantMode::Always).to_string();
    assert_eq!(Position::starting().to_fen(), initial);
}

#[test]
fn canonical_fen_round_trips() {
    for (fen, _) in perft_positions() {
        assert_eq!(Position::from_fen(fen).unwrap().to_fen(), fen);
    }
    reference_games(&Chess::default(), 1..=8, |position, _| {
        let fen = Fen::from_position(position, EnPassantMode::Always).to_string();
        assert_eq!(Position::from_fen(&fen).unwrap().to_fen(), fen);
    })
    .unwrap();
}

#[test]
fn leading_zeroes_in_the_halfmove_clock_are_accepted() {
    let (fen, _) = perft_positions().next().unwrap();
    let alternate = fen.replacen(" 0 1", " 00 1", 1);
    assert_ne!(alternate, fen);
    assert_eq!(Position::from_fen(&alternate).unwrap().to_fen(), fen);
}

#[test]
fn rejected_fens_identify_the_problem() {
    for (fen, problem) in [
        ("8/8/8/8/8/8/8/8 w - - 0", "six fields"),
        ("4k3/8/8/8/8/8/8/4Z3 w - - 0 1", "piece letter"),
        ("4k3/8/8/8/8/8/8/4K2 w - - 0 1", "eight files"),
        ("4k3/8/8/8/8/8/8/4K3/8 w - - 0 1", "eight ranks"),
        ("4k3/8/8/8/8/8/8/4K3 x - - 0 1", "side to move"),
        ("4k3/8/8/8/8/8/8/4K3 w X - 0 1", "K, Q, k, q"),
        ("4k3/8/8/8/8/8/8/4K2R w KK - 0 1", "duplicates"),
        ("4k3/8/8/8/8/8/8/4K3 w - z6 0 1", "invalid en passant"),
        ("4k3/8/8/8/8/8/8/4K3 w - a3 0 1", "wrong rank"),
        ("4k3/3p4/8/3p4/8/8/8/4K3 w - d6 0 1", "en passant"),
        ("4k3/8/8/8/4P3/4n3/8/4K3 b - e3 0 1", "en passant"),
        ("4k3/8/8/8/8/8/8/4K3 w - d6 0 1", "en passant"),
        ("4k3/8/8/3P4/8/8/8/4K3 w - d6 0 1", "en passant"),
        ("4k3/8/8/8/8/8/8/4K3 w - - +0 1", "halfmove clock"),
        ("4k3/8/8/8/8/8/8/4K3 w - - 0 +1", "fullmove number"),
        ("4k3/8/8/8/8/8/8/4K3 w - - x 1", "halfmove clock"),
        ("4k3/8/8/8/8/8/8/4K3 w - - 0 0", "fullmove number"),
        ("4k3/8/8/8/8/8/8/8 w - - 0 1", "one king for White"),
        ("k3k3/8/8/8/8/8/8/4K3 w - - 0 1", "one king for Black"),
        ("4k3/8/8/8/8/8/8/P3K3 w - - 0 1", "pawn on rank"),
        ("P3k3/8/8/8/8/8/8/4K3 w - - 0 1", "pawn on rank"),
        ("4k3/4K3/8/8/8/8/8/8 w - - 0 1", "adjacent kings"),
        ("4k3/8/8/8/8/8/8/4K3 w K - 0 1", "home squares"),
        ("4k3/8/8/8/8/8/8/3K3R w K - 0 1", "home squares"),
        ("4k3/8/8/8/8/8/8/4RK2 w - - 0 1", "in check"),
    ] {
        let error = Position::from_fen(fen).unwrap_err().to_string();
        assert!(error.contains(problem), "{fen}: {error}");
    }
}

#[test]
fn legal_moves_and_resulting_positions_match_reference_games() {
    let mut king_in_check = false;
    let mut king_beside_other_king = false;
    let mut played_en_passant = false;
    let mut played_castling = false;
    let mut played_promotion = false;
    let mut compare = |reference: &Chess, chosen: Option<shakmaty::Move>| {
        king_in_check |= reference.is_check();
        king_beside_other_king |= king_can_step_beside_other_king_on_rank_or_file(reference);
        compare_with_reference(reference).unwrap();
        if let Some(chosen) = chosen {
            played_en_passant |= chosen.is_en_passant();
            played_castling |= chosen.is_castle();
            played_promotion |= chosen.is_promotion();
        }
    };
    // Seed 17 gives the first castle. Seed 21 gives the first en passant capture.
    reference_games(&Chess::default(), 1..=21, &mut compare).unwrap();
    // Games from the starting position never let a king step beside the other
    // king on its rank or file. Games from the other published positions do.
    // The first published position is the starting position, so skip it.
    for (fen, _) in perft_positions().skip(1) {
        let start = fen
            .parse::<Fen>()
            .unwrap()
            .into_position::<Chess>(CastlingMode::Standard)
            .unwrap();
        reference_games(&start, SEEDS_FROM_PUBLISHED_POSITIONS, &mut compare).unwrap();
    }
    assert!(king_in_check, "no position had the king in check");
    assert!(
        king_beside_other_king,
        "no position let a king step beside the other king on its rank or file"
    );
    assert!(played_en_passant, "no en passant capture was played");
    assert!(played_castling, "no castle was played");
    assert!(played_promotion, "no promotion was played");
}

/// Few seeds, because `scripts/check` runs this on every commit.
const SEEDS_FROM_PUBLISHED_POSITIONS: RangeInclusive<u64> = 1..=4;

/// Whether the king of the side to move could step onto a square beside the
/// other king on its rank or file, a move the rules forbid.
fn king_can_step_beside_other_king_on_rank_or_file(reference: &Chess) -> bool {
    let kings = reference.board().kings();
    let (Some(king), Some(other_king)) = (
        (kings & reference.us()).first(),
        (kings & reference.them()).first(),
    ) else {
        return false;
    };
    shakmaty::attacks::king_attacks(king)
        .without(reference.us())
        .into_iter()
        .any(|square| {
            square.file().distance(other_king.file()) + square.rank().distance(other_king.rank())
                == 1
        })
}

fn compare_with_reference(reference: &Chess) -> Result<(), Box<dyn std::error::Error>> {
    let fen = Fen::from_position(reference, EnPassantMode::Always).to_string();
    let position = Position::from_fen(&fen)?;
    let mut actual: Vec<_> = position
        .legal_moves()
        .into_iter()
        .map(|movement| (uci_move(movement), movement))
        .collect();
    let mut expected: Vec<_> = reference
        .legal_moves()
        .into_iter()
        .map(|movement| (UciMove::from_standard(movement).to_string(), movement))
        .collect();
    actual.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
    expected.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
    assert_eq!(actual.len(), expected.len(), "legal move count at {fen}");

    for ((actual_uci, movement), (expected_uci, expected_move)) in actual.into_iter().zip(expected)
    {
        assert_eq!(actual_uci, expected_uci, "legal moves differ at {fen}");
        let next = reference.clone().play(expected_move)?;
        let expected_fen = Fen::from_position(&next, EnPassantMode::Always).to_string();
        assert_eq!(
            movement.play().to_fen(),
            expected_fen,
            "{fen} + {actual_uci}"
        );

        assert_eq!(
            position.legal_move(movement.from(), movement.to(), movement.promotion()),
            Some(movement),
            "move lookup at {fen} + {actual_uci}"
        );
    }
    Ok(())
}

#[test]
fn illegal_uci_moves_have_no_match() {
    let initial = Fen::from_position(&Chess::default(), EnPassantMode::Always).to_string();
    for (case, fen, from, to) in [
        ("three-square pawn advance", initial.as_str(), "e2", "e5"),
        (
            "missing promotion piece",
            "8/P7/8/8/8/6q1/5k2/7K w - - 0 1",
            "a7",
            "a8",
        ),
        (
            "en passant exposing the king",
            "k7/8/8/r4pPK/8/8/8/8 w - f6 0 1",
            "g5",
            "f6",
        ),
    ] {
        let reference = fen
            .parse::<Fen>()
            .unwrap()
            .into_position::<Chess>(CastlingMode::Standard)
            .unwrap();
        let notation = format!("{from}{to}");
        let uci = notation.parse::<UciMove>().unwrap();
        assert!(
            uci.to_move(&reference).is_err(),
            "{case}: {fen} + {notation}"
        );

        let position = Position::from_fen(fen).unwrap();
        assert!(
            position
                .legal_move(
                    Square::parse(from).unwrap(),
                    Square::parse(to).unwrap(),
                    None
                )
                .is_none(),
            "{case}: {fen} + {notation}"
        );
    }
}

fn uci_move(movement: Move) -> String {
    let promotion = match movement.promotion() {
        Some(Promotion::Knight) => "n",
        Some(Promotion::Bishop) => "b",
        Some(Promotion::Rook) => "r",
        Some(Promotion::Queen) => "q",
        None => "",
    };
    format!("{}{}{promotion}", movement.from(), movement.to())
}

#[test]
fn shallow_perft_matches_published_counts() {
    // Position 2 reaches castling and en passant at these depths.
    // Position 4 also reaches promotion.
    check_perft(&[3, 2, 3, 2, 3, 3], io::sink()).unwrap();
}

#[test]
#[ignore = "Set FISHER_PERFT_DEPTHS to six comma-separated depths and run in release mode"]
fn perft_at_selected_depths() {
    let depths: Vec<u8> = std::env::var("FISHER_PERFT_DEPTHS")
        .unwrap()
        .split(',')
        .map(str::parse)
        .collect::<Result<_, _>>()
        .unwrap();
    check_perft(&depths, io::stdout().lock()).unwrap();
}

fn check_perft(
    depths: &[u8],
    mut output: impl io::Write,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        perft_positions().count(),
        depths.len(),
        "depths per position"
    );
    for (index, ((fen, counts), &depth)) in perft_positions().zip(depths).enumerate() {
        let expected = published_perft_count(fen, counts, depth)?;
        let position = Position::from_fen(fen)?;
        let start = Instant::now();
        let actual = perft(&position, depth);
        writeln!(
            output,
            "Position {} D{depth} {actual} in {:.3}s",
            index + 1,
            start.elapsed().as_secs_f64()
        )?;
        assert_eq!(actual, expected, "perft at depth {depth} for {fen}");
    }
    Ok(())
}

fn perft(position: &Position, depth: u8) -> u128 {
    if depth == 0 {
        return 1;
    }
    let moves = position.legal_moves();
    if depth == 1 {
        return moves.into_iter().map(|_| 1_u128).sum();
    }
    moves
        .into_iter()
        .map(|movement| perft(&movement.play(), depth - 1))
        .sum()
}
