use std::error::Error;

use shakmaty::{CastlingMode, Chess, fen::Fen};

const PERFT: &str = include_str!("../../../../expectations/perft.epd");

pub(super) fn reference_position(fen: &str) -> Result<Chess, Box<dyn Error>> {
    Ok(fen.parse::<Fen>()?.into_position(CastlingMode::Standard)?)
}

pub(super) fn check_perft(
    depth: u8,
    nodes_at_depth: impl Fn(&str, u8) -> Result<u64, Box<dyn Error>>,
) -> Result<(), Box<dyn Error>> {
    for record in PERFT.lines().map(str::trim) {
        if record.is_empty() || record.starts_with('#') {
            continue;
        }
        let (fen, counts) = record.split_once(';').ok_or("missing perft counts")?;
        let fen = fen.trim();
        let mut expected = 0;
        for ply in 1..=depth {
            let label = format!("D{ply} ");
            let count = counts
                .split(';')
                .find_map(|entry| entry.trim().strip_prefix(&label))
                .ok_or_else(|| format!("missing perft count for {fen} at ply {ply}"))?;
            expected += count.parse::<u64>()?;
        }
        assert_eq!(
            nodes_at_depth(fen, depth)?,
            expected,
            "{fen} at depth {depth}"
        );
    }
    Ok(())
}
