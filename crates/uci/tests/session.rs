use std::io;

use deepening::{Score, SearchResult};
use rules::Square;

#[test]
fn session_reports_the_scripted_search() -> io::Result<()> {
    let mut output = Vec::new();
    uci::run_session(
        &b"position startpos moves e2e4\ngo\nquit\n"[..],
        &mut output,
        |position| {
            let reply = position
                .legal_move(
                    Square::parse("e7").unwrap(),
                    Square::parse("e5").unwrap(),
                    None,
                )
                .expect("the session passes the position after e2e4");
            SearchResult {
                depth: 1,
                score: Score::Centipawns(17),
                nodes: 23,
                principal_variation: vec![reply],
                chosen_move: Some(reply),
            }
        },
    )?;

    let output = String::from_utf8(output).map_err(io::Error::other)?;
    let lines: Vec<_> = output.lines().collect();
    let [info, bestmove] = lines.as_slice() else {
        return Err(io::Error::other(format!(
            "expected an info line and bestmove, got {output:?}"
        )));
    };
    assert!(info.starts_with("info depth 1 multipv 1 score cp 17 nodes 23 nps "));
    assert!(info.ends_with(" pv e7e5"));
    assert_eq!(*bestmove, "bestmove e7e5");
    Ok(())
}
