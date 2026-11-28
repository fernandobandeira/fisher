use rules::{Color, Move, Square};

#[derive(Default)]
pub(super) struct History {
    white: Vec<Origin>,
    black: Vec<Origin>,
}

struct Origin {
    square: Square,
    destinations: Vec<(Square, i32)>,
}

impl History {
    pub(super) fn destinations(&self, color: Color, origin: Square) -> &[(Square, i32)] {
        let origins = match color {
            Color::White => &self.white,
            Color::Black => &self.black,
        };
        origins
            .iter()
            .find(|entry| entry.square == origin)
            .map_or(&[], |entry| entry.destinations.as_slice())
    }

    pub(super) fn remember(&mut self, color: Color, movement: &Move, depth: u8) {
        let origins = match color {
            Color::White => &mut self.white,
            Color::Black => &mut self.black,
        };
        // Stockfish 17's positive butterfly history update. Fisher records
        // only quiet cutoffs, so there is no penalty for other moves.
        let bonus = (190 * i32::from(depth) - 108).min(1596);
        let from = movement.from();
        let to = movement.to();
        if let Some(origin) = origins.iter_mut().find(|entry| entry.square == from) {
            if let Some((_, score)) = origin.destinations.iter_mut().find(|entry| entry.0 == to) {
                *score += bonus - *score * bonus / 7183;
            } else {
                origin.destinations.push((to, bonus));
            }
        } else {
            origins.push(Origin {
                square: from,
                destinations: vec![(to, bonus)],
            });
        }
    }
}
