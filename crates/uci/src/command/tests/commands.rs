#![cfg(test)]

use super::super::*;
use rules::Square;

#[test]
fn ignored_commands_consume_the_rest_of_the_line() {
    for line in [
        "setoption name Path value quit",
        "debug on quit",
        "ponderhit quit",
        "register name quit",
    ] {
        assert_eq!(parse_command(line), Input::Ignored, "{line}");
    }
}

// The fields and searchmoves example come from the April 2006 UCI specification.
#[test]
fn parses_go_limits_and_ordered_searchmoves() -> Result<(), &'static str> {
    assert_eq!(
        parse_command(
            "go wtime 1000 btime 2000 winc 30 binc 40 movestogo 50 nodes 600 mate 7 \
             movetime 800 searchmoves e2e4 d2d4 depth 9"
        ),
        Input::Handled(Command::Go(GoParameters {
            wtime: Some(1000),
            btime: Some(2000),
            winc: Some(30),
            binc: Some(40),
            movestogo: Some(50),
            nodes: Some(600),
            mate: Some(7),
            movetime: Some(800),
            searchmoves: vec![uci_move("e2", "e4")?, uci_move("d2", "d4")?],
            depth: Some(9),
            ponder: false,
            infinite: false,
        })),
    );
    Ok(())
}

#[test]
fn parses_go_modes_independently() {
    assert_eq!(
        parse_command("go ponder"),
        Input::Handled(Command::Go(GoParameters {
            ponder: true,
            ..GoParameters::default()
        })),
    );
    assert_eq!(
        parse_command("go infinite"),
        Input::Handled(Command::Go(GoParameters {
            infinite: true,
            ..GoParameters::default()
        })),
    );
}

#[test]
fn missing_go_depth_preserves_infinite() {
    assert_eq!(
        parse_command("go depth infinite"),
        Input::Handled(Command::Go(GoParameters {
            infinite: true,
            ..GoParameters::default()
        })),
    );
}

#[test]
fn parses_position_sources_and_ordered_moves() -> Result<(), &'static str> {
    assert_eq!(
        parse_command("position startpos moves e2e4 e7e5"),
        Input::Handled(Command::Position(Ok(PositionCommand {
            source: PositionSource::Startpos,
            moves: vec![uci_move("e2", "e4")?, uci_move("e7", "e5")?],
        }))),
    );
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    assert_eq!(
        parse_command(&format!("position fen {fen} moves d2d4")),
        Input::Handled(Command::Position(Ok(PositionCommand {
            source: PositionSource::Fen(fen.to_owned()),
            moves: vec![uci_move("d2", "d4")?],
        }))),
    );
    Ok(())
}

fn uci_move(from: &str, to: &str) -> Result<UciMove, &'static str> {
    Ok(UciMove::Normal {
        from: Square::parse(from).ok_or("invalid expected origin square")?,
        to: Square::parse(to).ok_or("invalid expected destination square")?,
        promotion: None,
    })
}

#[test]
fn new_game_consumes_the_rest_of_the_line() {
    assert_eq!(
        parse_command("ucinewgame isready"),
        Input::Handled(Command::UciNewGame)
    );
}
