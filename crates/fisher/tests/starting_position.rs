use rules::Position;
use shakmaty::{Chess, Position as _, uci::UciMove};

#[test]
fn starting_position_has_every_legal_uci_move() {
    let mut expected: Vec<_> = Chess::default()
        .legal_moves()
        .into_iter()
        .map(|movement| UciMove::from_standard(movement).to_string())
        .collect();
    let mut actual: Vec<_> = Position::starting()
        .legal_moves()
        .into_iter()
        .map(|movement| uci::UciMove::from(movement).to_string())
        .collect();
    expected.sort_unstable();
    actual.sort_unstable();

    assert_eq!(actual, expected);
}
