use rules::Position;
use shakmaty::{Chess, EnPassantMode, Position as _, fen::Fen};

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

fn reference_games(mut visit: impl FnMut(&Chess)) -> Result<(), Box<dyn std::error::Error>> {
    for mut seed in 1_u64..=8 {
        let mut position = Chess::default();
        for _ in 0..128 {
            visit(&position);
            let moves = position.legal_moves();
            if moves.is_empty() {
                break;
            }
            seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            let count = u64::try_from(moves.len())?;
            let index = usize::try_from((seed >> 32) % count)?;
            let chosen = moves.get(index).copied().ok_or("missing reference move")?;
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
    reference_games(|position| {
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
    ] {
        let error = Position::from_fen(fen).unwrap_err().to_string();
        assert!(error.contains(problem), "{fen}: {error}");
    }
}
