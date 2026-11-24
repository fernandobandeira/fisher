use std::{io, sync::atomic::Ordering, time::Duration};

use deepening::{FinishedDepth, Limits, Score};
use rules::{Move, Position, Square};

#[test]
fn session_reports_the_scripted_search() -> io::Result<()> {
    let mut output = Vec::new();
    uci::run_session(
        &b"position startpos moves e2e4\ngo depth 2 nodes 440 movetime 50 infinite\nstop\n\
           position fen 7k/8/5K1Q/8/8/8/8/8 b - - 0 1\ngo nodes 0\nquit\n"[..],
        &mut output,
        "test",
        |request, report| {
            assert!(!request.stop.load(Ordering::Relaxed));
            if request.limits.nodes == Some(0) {
                assert_eq!(
                    request.limits,
                    Limits {
                        nodes: Some(0),
                        ..Limits::default()
                    }
                );
                // D20. This root has exactly one legal move, as in go-infinite.uci.
                return request.position.legal_moves().into_iter().next();
            }

            assert_eq!(
                request.limits,
                Limits {
                    depth: Some(2),
                    nodes: Some(440),
                    movetime: Some(Duration::from_millis(50)),
                    infinite: true,
                }
            );
            let chosen_move = legal_move(&request.position, "e7", "e5")?;
            let reply = legal_move(&chosen_move.play(), "g1", "f3")?;
            report(FinishedDepth {
                depth: 1,
                score: Score::Centipawns(17),
                nodes: 23,
                elapsed: Duration::from_millis(2),
                principal_variation: vec![chosen_move],
                chosen_move: Some(chosen_move),
            });
            report(FinishedDepth {
                depth: 2,
                score: Score::Centipawns(-6),
                nodes: 440,
                elapsed: Duration::from_millis(4),
                principal_variation: vec![chosen_move, reply],
                chosen_move: Some(chosen_move),
            });
            Some(chosen_move)
        },
    )?;

    assert_eq!(
        String::from_utf8(output).map_err(io::Error::other)?,
        concat!(
            "info depth 1 multipv 1 score cp 17 nodes 23 nps 11500 time 2 pv e7e5\n",
            "info depth 2 multipv 1 score cp -6 nodes 440 nps 110000 time 4 pv e7e5 g1f3\n",
            "bestmove e7e5\n",
            "bestmove h8g8\n",
        )
    );
    Ok(())
}

fn legal_move(position: &Position, from: &str, to: &str) -> Option<Move> {
    position.legal_move(Square::parse(from)?, Square::parse(to)?, None)
}
