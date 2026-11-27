//! Hand-written histories from fisher-8ie and D11, D20, D24 and D25,
//! read 2026-11-24. The two forced moves come from go-parameters.uci.
//! The finished-depth reports are scripted and do not run a search.

#[path = "support/search_history.rs"]
mod search_history;

use std::{io, time::Duration};

use deepening::{FinishedDepth, Score, SearchReport};
use search_history::{Event, Event::*, check, scripted_input};

const A: &str = "7k/8/5K1Q/8/8/8/8/8 b - - 0 1";
const B: &str = "7k/8/5K2/6Q1/8/8/8/8 b - - 0 1";
const POSITION_A: Event<'_> = Command("position fen 7k/8/5K1Q/8/8/8/8/8 b - - 0 1");
const POSITION_B: Event<'_> = Command("position fen 7k/8/5K2/6Q1/8/8/8/8 b - - 0 1");
const INVALID_POSITION: Event<'_> = Command("position fen 8/8/8/8/8/8/8/8 w - - 0 1");
const GO: Event<'_> = Command("go");
const INFINITE: Event<'_> = Command("go infinite");
const STOP: Event<'_> = Command("stop");
const QUIT: Event<'_> = Command("quit");
const READY: Event<'_> = Command("isready");
const READY_OK: Event<'_> = Response("readyok\n");
const MOVE_A: Event<'_> = Response("bestmove h8g8\n");
const MOVE_B: Event<'_> = Response("bestmove h8h7\n");
const INFO_A: Event<'_> =
    Response("info depth 1 multipv 1 score cp -900 nodes 1 nps 1000 time 1 pv h8g8\n");
const REJECT_POSITION: Event<'_> = Response("info string position rejected: waiting for stop\n");
const REJECT_GO: Event<'_> = Response("info string go rejected: waiting for stop\n");

#[expect(
    clippy::unwrap_used,
    reason = "the fixed FEN must have a sole legal move"
)]
fn forced_move(fen: &str) -> rules::Move {
    let moves = rules::Position::from_fen(fen).unwrap().legal_moves();
    assert_eq!(moves.len(), 1, "{fen}");
    moves.into_iter().next().unwrap()
}

fn best_move_report(fen: &str) -> Event<'static> {
    Report(SearchReport::BestMove(Some(forced_move(fen))))
}

fn finished_depth() -> Event<'static> {
    Report(SearchReport::FinishedDepth(FinishedDepth {
        depth: 1,
        nodes: 1,
        elapsed: Duration::from_millis(1),
        terminal_score: None,
        variations: vec![(Score::Centipawns(-900), vec![forced_move(A)])],
    }))
}

fn accepts(history: &[Event<'_>]) {
    assert_eq!(check(history), Ok(()), "{history:?}");
}

fn rejects(rule: u8, name: &str, history: &[Event<'_>]) {
    let result = check(history);
    assert_eq!(
        result.as_ref().err().map(|violation| violation.rule),
        Some(rule),
        "{name} should breach rule {rule}. Got {result:?} in {history:?}"
    );
}

#[test]
fn rule_1_one_bestmove_per_go() {
    accepts(&[POSITION_A, GO, best_move_report(A), MOVE_A, STOP, QUIT]);
    accepts(&[POSITION_A, GO, STOP, best_move_report(A), MOVE_A, QUIT]);
    rejects(
        1,
        "duplicate bestmove",
        &[
            POSITION_A,
            GO,
            best_move_report(A),
            MOVE_A,
            STOP,
            MOVE_A,
            QUIT,
        ],
    );
    rejects(
        1,
        "duplicate bestmove before the next search reports BestMove",
        &[
            POSITION_A,
            GO,
            best_move_report(A),
            STOP,
            GO,
            MOVE_A,
            MOVE_A,
            QUIT,
        ],
    );
    rejects(1, "bestmove without go", &[MOVE_A, QUIT]);
    rejects(
        1,
        "accepted go without bestmove",
        &[POSITION_A, GO, best_move_report(A)],
    );
}

#[test]
fn rule_2_infinite_holds_its_move() {
    accepts(&[
        POSITION_A,
        INFINITE,
        best_move_report(A),
        READY,
        READY_OK,
        STOP,
        MOVE_A,
        QUIT,
    ]);
    rejects(
        2,
        "early infinite bestmove",
        &[
            POSITION_A,
            INFINITE,
            best_move_report(A),
            MOVE_A,
            STOP,
            QUIT,
        ],
    );

    let terminal = Command("position fen 7k/6Q1/5K2/8/8/8/8/8 b - - 0 1");
    let null = Response("bestmove 0000\n");
    accepts(&[
        terminal.clone(),
        INFINITE,
        Report(SearchReport::BestMove(None)),
        STOP,
        null.clone(),
        QUIT,
    ]);
    rejects(
        2,
        "early infinite 0000",
        &[
            terminal,
            INFINITE,
            Report(SearchReport::BestMove(None)),
            null,
            STOP,
            QUIT,
        ],
    );
}

#[test]
fn rule_3_finished_depths_precede_bestmove() {
    accepts(&[
        POSITION_A,
        GO,
        STOP,
        finished_depth(),
        INFO_A,
        best_move_report(A),
        MOVE_A,
        QUIT,
    ]);
    rejects(
        3,
        "stop released bestmove before BestMove",
        &[
            POSITION_A,
            INFINITE,
            finished_depth(),
            INFO_A,
            STOP,
            MOVE_A,
            best_move_report(A),
            QUIT,
        ],
    );
    rejects(
        3,
        "late search info",
        &[
            POSITION_A,
            GO,
            STOP,
            best_move_report(A),
            MOVE_A,
            INFO_A,
            QUIT,
        ],
    );
    rejects(
        3,
        "missing finished depth",
        &[
            POSITION_A,
            GO,
            finished_depth(),
            STOP,
            best_move_report(A),
            MOVE_A,
            QUIT,
        ],
    );
    rejects(
        3,
        "info for an unfinished depth",
        &[
            POSITION_A,
            GO,
            STOP,
            INFO_A,
            best_move_report(A),
            MOVE_A,
            QUIT,
        ],
    );

    // Both e-pawn moves are legal. D11 still requires the one from the
    // last finished depth when stop interrupts the search.
    let starting = rules::Position::starting();
    let pawn_move = |target: &str| {
        starting.legal_moves().into_iter().find(|movement| {
            movement.from().to_string() == "e2" && movement.to().to_string() == target
        })
    };
    let depth = Report(SearchReport::FinishedDepth(FinishedDepth {
        depth: 1,
        nodes: 20,
        elapsed: Duration::from_millis(1),
        terminal_score: None,
        variations: vec![(Score::Centipawns(0), pawn_move("e4").into_iter().collect())],
    }));
    let info = Response("info depth 1 multipv 1 score cp 0 nodes 20 nps 20000 time 1 pv e2e4\n");
    rejects(
        3,
        "stop ignored the last finished depth",
        &[
            GO,
            depth,
            info,
            STOP,
            Report(SearchReport::BestMove(pawn_move("e3"))),
            Response("bestmove e2e3\n"),
            QUIT,
        ],
    );
}

#[test]
fn rule_4_idle_stop_is_silent() {
    accepts(&[STOP, READY, READY_OK, QUIT]);
    rejects(
        4,
        "idle stop added a readyok",
        &[STOP, READY, READY_OK, READY_OK, QUIT],
    );
    rejects(
        4,
        "idle stop wrote output",
        &[
            STOP,
            Response("info string no search\n"),
            READY,
            READY_OK,
            QUIT,
        ],
    );
}

#[test]
fn rule_5_bestmove_is_legal_in_the_accepted_position() {
    accepts(&[POSITION_B, GO, best_move_report(B), MOVE_B, QUIT]);
    rejects(
        5,
        "illegal bestmove",
        &[POSITION_A, GO, best_move_report(A), MOVE_B, QUIT],
    );
    rejects(
        5,
        "0000 with a legal move",
        &[
            POSITION_A,
            GO,
            best_move_report(A),
            Response("bestmove 0000\n"),
            QUIT,
        ],
    );
}

#[test]
fn rule_6_isready_does_not_end_a_search() {
    accepts(&[
        POSITION_A,
        INFINITE,
        READY,
        READY_OK,
        STOP,
        best_move_report(A),
        MOVE_A,
        QUIT,
    ]);
    rejects(
        6,
        "missing readyok during search",
        &[
            POSITION_A,
            INFINITE,
            READY,
            STOP,
            best_move_report(A),
            MOVE_A,
            QUIT,
        ],
    );
    rejects(
        6,
        "isready ended search before readyok",
        &[
            POSITION_A,
            INFINITE,
            READY,
            best_move_report(A),
            MOVE_A,
            READY_OK,
            QUIT,
        ],
    );
    rejects(
        6,
        "isready released infinite search after readyok",
        &[
            POSITION_A,
            INFINITE,
            READY,
            READY_OK,
            best_move_report(A),
            MOVE_A,
            QUIT,
        ],
    );
    rejects(6, "missing idle readyok", &[READY, QUIT]);
}

#[test]
fn rule_7_earlier_command_responses_precede_readyok() {
    accepts(&[
        POSITION_A,
        INFINITE,
        POSITION_B,
        READY,
        REJECT_POSITION,
        READY_OK,
        finished_depth(),
        INFO_A,
        STOP,
        best_move_report(A),
        MOVE_A,
        QUIT,
    ]);
    accepts(&[
        POSITION_A,
        INFINITE,
        POSITION_B,
        REJECT_POSITION,
        finished_depth(),
        INFO_A,
        READY,
        READY_OK,
        STOP,
        best_move_report(A),
        MOVE_A,
        QUIT,
    ]);
    rejects(
        7,
        "readyok overtook rejection",
        &[
            POSITION_A,
            INFINITE,
            POSITION_B,
            READY,
            READY_OK,
            REJECT_POSITION,
            STOP,
            best_move_report(A),
            MOVE_A,
            QUIT,
        ],
    );
}

#[test]
fn rule_8_reject_replacements_only_before_stop() {
    for (command, diagnostic) in [(POSITION_B, REJECT_POSITION), (GO, REJECT_GO)] {
        accepts(&[
            POSITION_A,
            INFINITE,
            command.clone(),
            diagnostic.clone(),
            best_move_report(A),
            STOP,
            MOVE_A,
            GO,
            best_move_report(A),
            MOVE_A,
            QUIT,
        ]);
        accepts(&[
            POSITION_A,
            INFINITE,
            best_move_report(A),
            command,
            diagnostic,
            STOP,
            MOVE_A,
            GO,
            best_move_report(A),
            MOVE_A,
            QUIT,
        ]);
    }
    accepts(&[
        POSITION_A,
        GO,
        POSITION_B,
        REJECT_POSITION,
        GO,
        REJECT_GO,
        best_move_report(A),
        MOVE_A,
        QUIT,
    ]);
    accepts(&[
        POSITION_A,
        GO,
        best_move_report(A),
        POSITION_B,
        GO,
        MOVE_A,
        best_move_report(B),
        MOVE_B,
        QUIT,
    ]);
    accepts(&[
        POSITION_A,
        INFINITE,
        STOP,
        POSITION_B,
        GO,
        best_move_report(A),
        MOVE_A,
        best_move_report(B),
        MOVE_B,
        QUIT,
    ]);
    accepts(&[
        POSITION_A,
        INFINITE,
        best_move_report(A),
        STOP,
        POSITION_B,
        GO,
        MOVE_A,
        best_move_report(B),
        MOVE_B,
        QUIT,
    ]);

    rejects(
        8,
        "rejected position took effect in the next search",
        &[
            POSITION_A,
            INFINITE,
            POSITION_B,
            REJECT_POSITION,
            best_move_report(A),
            STOP,
            MOVE_A,
            GO,
            best_move_report(B),
            MOVE_B,
            QUIT,
        ],
    );
    rejects(
        8,
        "rejected go started another search",
        &[
            POSITION_A,
            INFINITE,
            GO,
            REJECT_GO,
            best_move_report(A),
            STOP,
            MOVE_A,
            best_move_report(A),
            MOVE_A,
            QUIT,
        ],
    );
    rejects(
        8,
        "position and go sent after stop were rejected",
        &[
            POSITION_A,
            INFINITE,
            STOP,
            POSITION_B,
            GO,
            REJECT_POSITION,
            REJECT_GO,
            best_move_report(A),
            MOVE_A,
            QUIT,
        ],
    );
    rejects(
        8,
        "go sent after stop was rejected",
        &[
            POSITION_A,
            INFINITE,
            STOP,
            POSITION_B,
            GO,
            best_move_report(A),
            MOVE_A,
            REJECT_GO,
            QUIT,
        ],
    );
}

#[test]
fn rule_9_quit_and_end_of_input_release_the_move_then_exit() {
    for exit in [QUIT, EndOfInput] {
        accepts(&[
            POSITION_A,
            INFINITE,
            best_move_report(A),
            exit.clone(),
            MOVE_A,
        ]);
        accepts(&[
            POSITION_A,
            INFINITE,
            exit.clone(),
            best_move_report(A),
            MOVE_A,
        ]);
        accepts(&[POSITION_A, GO, exit.clone(), best_move_report(A), MOVE_A]);
        accepts(&[POSITION_A, GO, best_move_report(A), exit.clone(), MOVE_A]);
        accepts(&[
            POSITION_A,
            GO,
            STOP,
            exit.clone(),
            best_move_report(A),
            MOVE_A,
        ]);
        accepts(std::slice::from_ref(&exit));
        rejects(
            9,
            "exit omitted bestmove",
            &[POSITION_A, GO, exit.clone(), best_move_report(A)],
        );
        rejects(
            9,
            "output after exit bestmove",
            &[
                POSITION_A,
                INFINITE,
                best_move_report(A),
                exit,
                MOVE_A,
                READY_OK,
            ],
        );
    }
}

#[test]
fn rule_10_responses_are_whole_lines() {
    accepts(&[
        Command("uci"),
        Response("id name fisher test\n"),
        Response("id author Fernando Bandeira\n"),
        Response("option name MultiPV type spin default 1 min 1 max 255\n"),
        Response("uciok\n"),
        READY,
        READY_OK,
        QUIT,
    ]);
    rejects(10, "missing newline", &[READY, Response("readyok"), QUIT]);
    rejects(
        10,
        "two responses in one line",
        &[
            POSITION_A,
            GO,
            READY,
            best_move_report(A),
            Response("bestmove h8g8readyok\n"),
            QUIT,
        ],
    );
    rejects(
        10,
        "two responses in one event",
        &[READY, READY, Response("readyok\nreadyok\n"), QUIT],
    );
}

#[test]
fn commands_after_exit_are_ignored() {
    for exit in [QUIT, EndOfInput] {
        accepts(&[
            POSITION_A,
            GO,
            best_move_report(A),
            exit.clone(),
            READY,
            MOVE_A,
        ]);
        for go in [GO, INFINITE] {
            accepts(&[
                POSITION_A,
                go,
                STOP,
                exit.clone(),
                READY,
                best_move_report(A),
                MOVE_A,
            ]);
        }
        accepts(&[exit, READY, GO, POSITION_B]);
    }
}

#[test]
fn invalid_idle_position_is_reported() {
    for (command, diagnostic) in [
        (
            Command("position fen 8/8/8/8/8/8/8/8 w - - 0 1"),
            Response("info string position rejected: invalid FEN: no king\n"),
        ),
        (
            Command("position startpos moves e2e5"),
            Response("info string position rejected: illegal UCI move e2e5\n"),
        ),
    ] {
        accepts(&[command.clone(), diagnostic.clone(), READY, READY_OK, QUIT]);
        accepts(&[command.clone(), READY, diagnostic.clone(), READY_OK, QUIT]);
        accepts(&[
            STOP,
            command.clone(),
            READY,
            diagnostic.clone(),
            READY_OK,
            QUIT,
        ]);
        accepts(&[STOP, READY, command, READY_OK, diagnostic, QUIT]);
    }
}

#[test]
fn readiness_uses_stockfish_whitespace() {
    accepts(&[Command("isready\u{a0}x"), QUIT]);
    for command in [
        "isready\tx",
        "isready\tisready",
        "isready\x0bx",
        "isready\x0bisready",
    ] {
        accepts(&[Command(command), READY_OK, QUIT]);
    }
}

#[test]
fn deferred_readiness_uses_stockfish_whitespace() {
    rejects(
        6,
        "a deferred isready with a vertical tab requires readyok",
        &[GO, STOP, Command("isready\x0bx"), QUIT],
    );
    rejects(
        8,
        "a deferred quit with a vertical tab hides later readiness",
        &[GO, STOP, Command("quit\x0bx"), Command("isready\tx")],
    );
}

#[test]
fn readiness_after_an_unknown_prefix() {
    let ready = Command("stop\u{a0}x isready");
    rejects(
        6,
        "missing readyok after an unknown prefix",
        &[ready.clone(), QUIT],
    );
    accepts(&[ready, READY_OK, QUIT]);
    for ignored in ["setoption", "debug", "ponderhit", "register", "ucinewgame"] {
        let command = format!("stop\u{a0}x {ignored} isready");
        accepts(&[Command(&command), QUIT]);
    }
}

#[test]
fn deferred_readiness_after_an_unknown_prefix() {
    let ready = Command("stop\u{a0}x isready");
    rejects(
        6,
        "missing deferred readiness behind stop",
        &[POSITION_A, GO, STOP, ready.clone(), QUIT],
    );
    rejects(
        6,
        "missing deferred readyok after bestmove",
        &[
            POSITION_A,
            GO,
            STOP,
            ready.clone(),
            best_move_report(A),
            MOVE_A,
            QUIT,
        ],
    );
    accepts(&[
        POSITION_A,
        GO,
        STOP,
        ready,
        best_move_report(A),
        MOVE_A,
        READY_OK,
        QUIT,
    ]);
}

#[test]
fn deferred_quit_after_an_unknown_prefix() {
    let quit = Command("stop\u{a0}x quit");
    rejects(
        8,
        "missing bestmove when a deferred quit hides later readiness",
        &[POSITION_A, GO, STOP, quit.clone(), READY],
    );
    let mut history = vec![
        POSITION_A,
        GO,
        STOP,
        quit,
        READY,
        best_move_report(A),
        MOVE_A,
    ];
    accepts(&history);
    history.push(READY_OK);
    rejects(9, "unsolicited readyok after a deferred quit", &history);
}

#[test]
fn a_joined_en_passant_field_is_rejected_by_the_history_checker() {
    for command in [
        "position fen 4k3/8/8/8/8/8/8/4K2R w K -\u{a0}0 1",
        "position fen 4k3/8/8/8/8/8/8/4K2R w K -\u{a0}7 42",
    ] {
        accepts(&[
            POSITION_A,
            Command(command),
            Response("info string position rejected: invalid FEN: invalid en passant square\n"),
            GO,
            best_move_report(A),
            MOVE_A,
            QUIT,
        ]);
    }
}

#[test]
fn readiness_on_either_side_of_finite_bestmove() {
    accepts(&[
        POSITION_A,
        GO,
        best_move_report(A),
        READY,
        READY_OK,
        MOVE_A,
        QUIT,
    ]);
    accepts(&[
        POSITION_A,
        GO,
        best_move_report(A),
        READY,
        MOVE_A,
        READY_OK,
        QUIT,
    ]);
    rejects(
        6,
        "isready before finite BestMove must be answered before bestmove",
        &[
            POSITION_A,
            GO,
            READY,
            best_move_report(A),
            MOVE_A,
            READY_OK,
            QUIT,
        ],
    );
}

#[test]
fn replacements_on_either_side_of_finite_bestmove() {
    for (command, diagnostic) in [(POSITION_B, REJECT_POSITION), (GO, REJECT_GO)] {
        accepts(&[
            POSITION_A,
            GO,
            best_move_report(A),
            command,
            diagnostic,
            MOVE_A,
            QUIT,
        ]);
    }
    accepts(&[
        POSITION_A,
        GO,
        best_move_report(A),
        MOVE_A,
        POSITION_B,
        GO,
        best_move_report(B),
        MOVE_B,
        QUIT,
    ]);
}

#[test]
fn fen_fields_end_before_the_move_list() {
    for position in [
        "position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - moves e2e4",
        "position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 moves e2e4",
        "position fen rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 moves e2e4",
    ] {
        let after_e4 = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1";
        let movement = rules::Position::from_fen(after_e4)
            .ok()
            .and_then(|position| {
                position.legal_moves().into_iter().find(|movement| {
                    movement.from().to_string() == "e7" && movement.to().to_string() == "e5"
                })
            });
        assert!(movement.is_some());
        accepts(&[
            Command(position),
            GO,
            Report(SearchReport::BestMove(movement)),
            Response("bestmove e7e5\n"),
            QUIT,
        ]);
    }
}

#[test]
fn fen_clocks_follow_the_engines_existing_decimal_rules() -> io::Result<()> {
    // Expected acceptance comes from the engine's existing behavior at dec2a5c.
    let fen = "7k/8/5K2/6Q1/8/8/8/8 w - -";
    for (clocks, accepted) in [
        ("", true),
        (" 0", true),
        (" 00017 0001", true),
        (" 4294967296 4294967296", true),
        (" 18446744073709551615 18446744073709551615", true),
        (" 3+3", false),
        (" 3+3 1", false),
        (" 0 3+3", false),
        (" banana", false),
        (" banana 1", false),
        (" 0 banana", false),
        (" ١ 1", false),
        (" 3_3 1", false),
        (" +3", false),
        (" +3 1", false),
        (" 0 +3", false),
        (" -1", false),
        (" -1 1", false),
        (" 0 -1", false),
        (" 18446744073709551616", false),
        (" 18446744073709551616 1", false),
        (" 0 18446744073709551616", false),
        (" 0 0", false),
        (" 0 1 extra", false),
    ] {
        let command = format!("position fen {fen}{clocks}");
        let input = format!("{command}\nisready\nquit\n").into_bytes();
        let mut output = Vec::new();
        uci::run_session(io::Cursor::new(input), &mut output, "test", |_, _| {})?;
        let output = String::from_utf8(output).map_err(io::Error::other)?;
        assert_eq!(output == "readyok\n", accepted, "{command}: {output:?}");
        if accepted {
            accepts(&[Command(&command), READY, READY_OK, QUIT]);
        } else {
            assert!(
                output.starts_with("info string position rejected: invalid FEN: "),
                "{command}: {output:?}"
            );
            let diagnostic = output.lines().next().unwrap_or_default();
            let diagnostic = format!("{diagnostic}\n");
            accepts(&[
                Command(&command),
                Response(&diagnostic),
                READY,
                READY_OK,
                QUIT,
            ]);
        }
    }
    Ok(())
}

// D46 and fisher-8hc answer 4 specify the MultiPV command responses.
const SET_MULTIPV: Event<'_> = Command("setoption name mUlTiPv value 2");
const ZERO_MULTIPV: Event<'_> = Command("setoption name MultiPV value 0");
const UNKNOWN_OPTION: Event<'_> = Command("setoption name Path value stop");
const REJECT_SETOPTION: Event<'_> = Response("info string setoption rejected: waiting for stop\n");
const INVALID_MULTIPV: Event<'_> =
    Response("info string setoption rejected: MultiPV requires an integer from 1 to 255\n");

#[test]
fn multipv_setoption_uses_stockfish_whitespace() {
    for command in [
        "setoption name MultiPV value 2",
        "setoption name MultiPV value\t2",
        "setoption name MultiPV value\x0b2",
    ] {
        // While searching the checker must recognize the option and
        // require the busy diagnostic. While idle a good value is silent.
        accepts(&[
            POSITION_A,
            GO,
            Command(command),
            REJECT_SETOPTION,
            best_move_report(A),
            MOVE_A,
            QUIT,
        ]);
        accepts(&[Command(command), READY, READY_OK, QUIT]);
    }
    let malformed = Command("setoption name MultiPV value \u{a0}2");
    accepts(&[malformed.clone(), INVALID_MULTIPV, READY, READY_OK, QUIT]);
    rejects(
        7,
        "the joined value requires a diagnostic",
        &[malformed, READY, READY_OK, QUIT],
    );
}

#[test]
fn setoption_diagnostics_require_the_matching_command_and_state() {
    for (name, history) in [
        (
            "busy rejection while idle",
            [SET_MULTIPV, REJECT_SETOPTION, QUIT],
        ),
        (
            "bad-value diagnostic for a good value",
            [SET_MULTIPV, INVALID_MULTIPV, QUIT],
        ),
        (
            "diagnostic for an undeclared option",
            [UNKNOWN_OPTION, INVALID_MULTIPV, QUIT],
        ),
    ] {
        rejects(10, name, &history);
    }
}

#[test]
fn setoption_diagnostics_must_precede_readyok() {
    rejects(
        7,
        "missing diagnostic for value 0",
        &[ZERO_MULTIPV, READY, READY_OK, QUIT],
    );
    for (name, history) in [
        (
            "missing rejection while searching",
            vec![
                POSITION_A,
                INFINITE,
                SET_MULTIPV,
                READY,
                READY_OK,
                STOP,
                best_move_report(A),
                MOVE_A,
                QUIT,
            ],
        ),
        (
            "missing rejection while holding the move",
            vec![
                POSITION_A,
                INFINITE,
                best_move_report(A),
                SET_MULTIPV,
                READY,
                READY_OK,
                STOP,
                MOVE_A,
                QUIT,
            ],
        ),
    ] {
        rejects(7, name, &history);
    }
}

#[test]
fn multipv_history_requires_exactly_the_reported_variations() {
    let pawn_move = |target| {
        rules::Position::starting()
            .legal_moves()
            .into_iter()
            .find(|movement| movement.to().to_string() == target)
    };
    let best_move = pawn_move("e4");
    let report = Report(SearchReport::FinishedDepth(FinishedDepth {
        depth: 1,
        nodes: 20,
        elapsed: Duration::from_millis(1),
        terminal_score: None,
        variations: vec![
            (Score::Centipawns(0), best_move.into_iter().collect()),
            (Score::Centipawns(0), pawn_move("d4").into_iter().collect()),
        ],
    }));
    let first = Response("info depth 1 multipv 1 score cp 0 nodes 20 nps 20000 time 1 pv e2e4\n");
    let second = Response("info depth 1 multipv 2 score cp 0 nodes 20 nps 20000 time 1 pv d2d4\n");
    let extra = Response("info depth 1 multipv 3 score cp 0 nodes 20 nps 20000 time 1 pv g1f3\n");
    let best_move = Report(SearchReport::BestMove(best_move));
    let bestmove = Response("bestmove e2e4\n");

    let history = |variations: &[Event<'static>]| {
        let mut events = vec![
            SET_MULTIPV,
            ZERO_MULTIPV,
            INVALID_MULTIPV,
            INFINITE,
            report.clone(),
        ];
        events.extend_from_slice(variations);
        events.extend([
            best_move.clone(),
            UNKNOWN_OPTION,
            SET_MULTIPV,
            REJECT_SETOPTION,
            STOP,
            bestmove.clone(),
            QUIT,
        ]);
        events
    };
    accepts(&history(&[first.clone(), second.clone()]));
    rejects(
        3,
        "bestmove before variation 2",
        &history(std::slice::from_ref(&first)),
    );
    rejects(
        3,
        "unreported variation 3",
        &history(&[first, second, extra]),
    );
}

#[test]
fn history_rejects_a_searched_depth_without_a_variation() {
    for variations in [Vec::new(), vec![(Score::Centipawns(-900), Vec::new())]] {
        let depth = Report(SearchReport::FinishedDepth(FinishedDepth {
            depth: 1,
            nodes: 1,
            elapsed: Duration::from_millis(1),
            terminal_score: None,
            variations,
        }));
        rejects(3, "invalid searched depth report", &[POSITION_A, GO, depth]);
    }
}

#[test]
fn command_responses_precede_bestmove() {
    for (rule, name, history) in [
        (
            8,
            "B1: late rejection for position recorded after BestMove",
            vec![
                POSITION_A,
                GO,
                best_move_report(A),
                POSITION_B,
                MOVE_A,
                REJECT_POSITION,
                QUIT,
            ],
        ),
        (
            8,
            "B2: rejection for position recorded after bestmove",
            vec![
                POSITION_A,
                GO,
                best_move_report(A),
                MOVE_A,
                POSITION_B,
                REJECT_POSITION,
                QUIT,
            ],
        ),
        (
            8,
            "B3: late rejection for position recorded before BestMove",
            vec![
                POSITION_A,
                GO,
                POSITION_B,
                best_move_report(A),
                MOVE_A,
                REJECT_POSITION,
                QUIT,
            ],
        ),
        (
            8,
            "B4: late rejection for position recorded before stop",
            vec![
                POSITION_A,
                INFINITE,
                POSITION_B,
                STOP,
                best_move_report(A),
                MOVE_A,
                REJECT_POSITION,
                QUIT,
            ],
        ),
        (
            6,
            "B5: late readyok for isready recorded before stop",
            vec![
                POSITION_A,
                INFINITE,
                READY,
                STOP,
                best_move_report(A),
                MOVE_A,
                READY_OK,
                QUIT,
            ],
        ),
        (
            8,
            "B6: late rejection for go recorded before stop",
            vec![
                POSITION_A,
                INFINITE,
                GO,
                STOP,
                best_move_report(A),
                MOVE_A,
                REJECT_GO,
                QUIT,
            ],
        ),
        (
            8,
            "B31: late waiting-for-stop rejection for invalid FEN",
            vec![
                POSITION_A,
                GO,
                best_move_report(A),
                INVALID_POSITION,
                MOVE_A,
                REJECT_POSITION,
                QUIT,
            ],
        ),
    ] {
        rejects(rule, name, &history);
    }
    for exit in [QUIT, EndOfInput] {
        rejects(
            6,
            "exit bestmove overtook readyok for an earlier isready",
            &[
                POSITION_A,
                INFINITE,
                READY,
                exit,
                best_move_report(A),
                MOVE_A,
                READY_OK,
            ],
        );
    }
}

// Shows the scripted input driving the public session entry. While the search
// runs on the command thread it cannot detect an early acknowledgement; see the
// recording contract in `support/search_history.rs`.
#[test]
fn scripted_input_drives_the_session_entry() -> io::Result<()> {
    let (input, commands) = scripted_input();
    let Command(position) = POSITION_A else {
        unreachable!()
    };
    let depth = FinishedDepth {
        depth: 1,
        nodes: 1,
        elapsed: Duration::from_millis(1),
        terminal_score: None,
        variations: vec![(Score::Centipawns(-900), vec![forced_move(A)])],
    };
    let best_move = depth
        .variations
        .first()
        .and_then(|(_, moves)| moves.first())
        .copied();
    let mut output = Vec::new();
    std::thread::scope(|scope| -> io::Result<()> {
        let (controller, commands_for_search) = std::sync::mpsc::channel();
        let feeder = scope.spawn(move || -> io::Result<()> {
            commands.send_line(position)?;
            commands.send_line("go infinite")?;
            controller
                .send(commands)
                .map_err(|_| io::Error::other("search did not start"))
        });
        uci::run_session(input, &mut output, "test", move |request, report| {
            assert!(request.limits.infinite);
            let commands = commands_for_search
                .recv_timeout(search_history::HANG_TIMEOUT)
                .unwrap();
            commands.send_line("isready").unwrap();
            report(SearchReport::FinishedDepth(depth.clone()));
            commands.send_line("stop").unwrap();
            report(SearchReport::BestMove(best_move));
            commands.send_line("quit").unwrap();
        })?;
        feeder.join().unwrap()
    })?;

    let text = std::str::from_utf8(&output).map_err(io::Error::other)?;
    let mut history = vec![
        POSITION_A,
        INFINITE,
        READY,
        finished_depth(),
        STOP,
        best_move_report(A),
        QUIT,
    ];
    history.extend(text.split_inclusive('\n').map(Response));
    accepts(&history);
    Ok(())
}

// The fix-check review model uses a single handler, a reader, and a worker.
// Enumerate their schedules independently of the checker. A script is bounded
// by input and report actions. Delivery and handling do not spend that budget.
mod model {
    use std::collections::{HashSet, VecDeque};

    use super::*;

    const MAX_ACTIONS: usize = 4;
    const STOP_ACTIONS: usize = 5;
    const INPUTS: [Action; 8] = [
        Action::Position,
        Action::Invalid,
        Action::Go,
        Action::Infinite,
        Action::Stop,
        Action::Ready,
        Action::Quit,
        Action::Eof,
    ];

    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    enum Action {
        Position,
        Invalid,
        Go,
        Infinite,
        Stop,
        Ready,
        Quit,
        Eof,
    }

    #[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
    enum Root {
        #[default]
        A,
        B,
    }

    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    enum ReportKind {
        Depth,
        BestMove,
    }

    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    enum Message {
        Input(Action),
        Report(ReportKind, Root),
    }

    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    enum Line {
        Info(Root),
        Bestmove(Root),
        Ready,
        PositionRejected,
        InvalidPosition,
        GoRejected,
    }

    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    enum Observation {
        Message(Message),
        Line(Line),
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum Engine {
        Correct,
        LateRejection,
        LateReady,
        IdleStopRepeats,
        QuitDropsMove,
        ReadyReleasesMove,
        NoStopBarrier,
        ReadyAfterQuit,
    }

    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    enum Phase {
        Working,
        Held,
        Stopping,
        Exiting,
    }

    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    struct Work {
        root: Root,
        infinite: bool,
        phase: Phase,
        depth_reported: bool,
        best_move_reported: bool,
    }

    #[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
    enum InputState {
        #[default]
        Open,
        QuitRecorded,
        Closed,
    }

    #[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
    struct Schedule {
        actions: usize,
        history: Vec<Observation>,
        input: InputState,
        stop_during_search: bool,
        worker: VecDeque<Message>,
        channel: VecDeque<Message>,
        deferred: VecDeque<Action>,
        late: Vec<Line>,
        root: Root,
        previous_move: Option<Root>,
        work: Option<Work>,
        exited: bool,
    }

    impl Schedule {
        fn record(&mut self, message: Message) {
            self.actions += 1;
            self.history.push(Observation::Message(message));
        }

        fn write(&mut self, line: Line) {
            self.history.push(Observation::Line(line));
        }

        fn answer(&mut self, engine: Engine, line: Line) {
            let delay = match line {
                Line::Ready => engine == Engine::LateReady,
                Line::PositionRejected | Line::GoRejected => engine == Engine::LateRejection,
                Line::Info(_) | Line::Bestmove(_) | Line::InvalidPosition => false,
            };
            if delay && self.work.is_some() {
                self.late.push(line);
            } else {
                self.write(line);
            }
        }

        fn release(&mut self, engine: Engine, work: Work) {
            self.write(Line::Bestmove(work.root));
            self.previous_move = Some(work.root);
            self.work = None;
            self.history
                .extend(self.late.drain(..).map(Observation::Line));
            self.exited = work.phase == Phase::Exiting;
            while !self.exited
                && !self
                    .work
                    .is_some_and(|work| matches!(work.phase, Phase::Stopping | Phase::Exiting))
            {
                let Some(command) = self.deferred.pop_front() else {
                    break;
                };
                self.handle(engine, Message::Input(command));
            }
        }

        fn handle(&mut self, engine: Engine, message: Message) {
            match message {
                Message::Input(command) => self.command(engine, command),
                Message::Report(ReportKind::Depth, root) => self.write(Line::Info(root)),
                Message::Report(ReportKind::BestMove, _) => {
                    let Some(mut work) = self.work else {
                        unreachable!()
                    };
                    if work.infinite && work.phase == Phase::Working {
                        work.phase = Phase::Held;
                        self.work = Some(work);
                    } else {
                        self.release(engine, work);
                    }
                }
            }
        }

        fn command(&mut self, engine: Engine, command: Action) {
            if self.exited {
                if engine == Engine::ReadyAfterQuit && command == Action::Ready {
                    self.write(Line::Ready);
                }
                return;
            }
            match self.work {
                Some(work) => self.command_while_searching(engine, command, work),
                None => self.command_while_idle(engine, command),
            }
        }

        fn command_while_searching(&mut self, engine: Engine, command: Action, work: Work) {
            if work.phase == Phase::Exiting
                || (work.phase == Phase::Stopping && engine != Engine::NoStopBarrier)
            {
                self.deferred.push_back(command);
                return;
            }
            match command {
                Action::Position | Action::Invalid => self.answer(engine, Line::PositionRejected),
                Action::Go | Action::Infinite => self.answer(engine, Line::GoRejected),
                Action::Ready => {
                    self.answer(engine, Line::Ready);
                    if engine == Engine::ReadyReleasesMove && work.phase == Phase::Held {
                        self.release(engine, work);
                    }
                }
                Action::Quit | Action::Eof if engine == Engine::QuitDropsMove => {
                    self.exited = true;
                }
                Action::Stop | Action::Quit | Action::Eof => self.end_search(engine, command, work),
            }
        }

        fn end_search(&mut self, engine: Engine, command: Action, mut work: Work) {
            let held = work.phase == Phase::Held;
            work.phase = if command == Action::Stop {
                Phase::Stopping
            } else {
                Phase::Exiting
            };
            if held {
                self.release(engine, work);
            } else {
                self.work = Some(work);
            }
        }

        fn command_while_idle(&mut self, engine: Engine, command: Action) {
            match command {
                Action::Position => self.root = Root::B,
                Action::Invalid => self.write(Line::InvalidPosition),
                Action::Go | Action::Infinite => {
                    self.work = Some(Work {
                        root: self.root,
                        infinite: command == Action::Infinite,
                        phase: Phase::Working,
                        depth_reported: false,
                        best_move_reported: false,
                    });
                }
                Action::Ready => self.write(Line::Ready),
                Action::Quit | Action::Eof => self.exited = true,
                Action::Stop if engine == Engine::IdleStopRepeats => {
                    if let Some(root) = self.previous_move {
                        self.write(Line::Bestmove(root));
                    }
                }
                Action::Stop => {}
            }
        }

        fn record_input(&mut self, command: Action) {
            self.record(Message::Input(command));
            match command {
                Action::Quit => self.input = InputState::QuitRecorded,
                Action::Eof => self.input = InputState::Closed,
                Action::Position
                | Action::Invalid
                | Action::Go
                | Action::Infinite
                | Action::Stop
                | Action::Ready => {}
            }
            self.stop_during_search |=
                command == Action::Stop && self.work.is_some_and(|work| !work.best_move_reported);
            self.channel.push_back(Message::Input(command));
        }

        fn record_report(&mut self, kind: ReportKind, mut work: Work) {
            let message = Message::Report(kind, work.root);
            self.record(message);
            self.worker.push_back(message);
            match kind {
                ReportKind::Depth => work.depth_reported = true,
                ReportKind::BestMove => work.best_move_reported = true,
            }
            self.work = Some(work);
        }
    }

    type Histories = HashSet<Vec<Observation>>;

    fn explore(
        engine: Engine,
        schedule: Schedule,
        seen: &mut HashSet<Schedule>,
        histories: &mut Histories,
    ) {
        if !seen.insert(schedule.clone()) {
            return;
        }

        // D24 needs one more action to observe a command after stop
        // before the worker ends, followed by exit.
        let limit = if schedule.stop_during_search {
            STOP_ACTIONS
        } else {
            MAX_ACTIONS
        };
        if schedule.actions < limit {
            explore_inputs(engine, &schedule, limit, seen, histories);
            explore_reports(engine, &schedule, limit, seen, histories);
        }
        explore_deliveries(engine, &schedule, seen, histories);
        if schedule.exited && schedule.worker.is_empty() && schedule.channel.is_empty() {
            histories.insert(schedule.history);
        }
    }

    fn explore_inputs(
        engine: Engine,
        schedule: &Schedule,
        limit: usize,
        seen: &mut HashSet<Schedule>,
        histories: &mut Histories,
    ) {
        if schedule.input == InputState::Closed {
            return;
        }
        for command in INPUTS {
            if schedule.actions + 1 < limit
                || schedule.input != InputState::Open
                || matches!(command, Action::Quit | Action::Eof)
            {
                let mut next = schedule.clone();
                next.record_input(command);
                explore(engine, next, seen, histories);
            }
        }
    }

    fn explore_reports(
        engine: Engine,
        schedule: &Schedule,
        limit: usize,
        seen: &mut HashSet<Schedule>,
        histories: &mut Histories,
    ) {
        if !schedule.exited
            && (schedule.actions + 1 < limit || schedule.input != InputState::Open)
            && let Some(work) = schedule.work.filter(|work| !work.best_move_reported)
        {
            for kind in [ReportKind::Depth, ReportKind::BestMove] {
                if kind == ReportKind::BestMove || !work.depth_reported {
                    let mut next = schedule.clone();
                    next.record_report(kind, work);
                    explore(engine, next, seen, histories);
                }
            }
        }
    }

    fn explore_deliveries(
        engine: Engine,
        schedule: &Schedule,
        seen: &mut HashSet<Schedule>,
        histories: &mut Histories,
    ) {
        // Delivery is unobservable. Delivering BestMove just before or just
        // after recording a command enumerates both channel orders. Commands
        // already recorded when the worker reports are already enqueued.
        if !schedule.worker.is_empty() {
            let mut next = schedule.clone();
            if let Some(message) = next.worker.pop_front() {
                next.channel.push_back(message);
            }
            explore(engine, next, seen, histories);
        }
        if !schedule.channel.is_empty() {
            let mut next = schedule.clone();
            if let Some(message) = next.channel.pop_front()
                && (!next.exited || matches!(message, Message::Input(_)))
            {
                next.handle(engine, message);
            }
            explore(engine, next, seen, histories);
        }
    }

    fn generate(engine: Engine) -> Histories {
        let mut histories = HashSet::new();
        explore(
            engine,
            Schedule::default(),
            &mut HashSet::new(),
            &mut histories,
        );
        histories
    }

    fn event(observation: Observation, moves: &[rules::Move; 2]) -> Event<'static> {
        match observation {
            Observation::Message(Message::Input(command)) => match command {
                Action::Position => POSITION_B,
                Action::Invalid => INVALID_POSITION,
                Action::Go => GO,
                Action::Infinite => INFINITE,
                Action::Stop => STOP,
                Action::Ready => READY,
                Action::Quit => QUIT,
                Action::Eof => EndOfInput,
            },
            Observation::Message(Message::Report(kind, root)) => {
                let movement = match root {
                    Root::A => moves[0],
                    Root::B => moves[1],
                };
                match kind {
                    ReportKind::Depth => Report(SearchReport::FinishedDepth(FinishedDepth {
                        depth: 1,
                        nodes: 1,
                        elapsed: Duration::from_millis(1),
                        terminal_score: None,
                        variations: vec![(Score::Centipawns(-900), vec![movement])],
                    })),
                    ReportKind::BestMove => Report(SearchReport::BestMove(Some(movement))),
                }
            }
            Observation::Line(line) => match line {
                Line::Info(Root::A) => INFO_A,
                Line::Info(Root::B) => Response(
                    "info depth 1 multipv 1 score cp -900 nodes 1 nps 1000 time 1 pv h8h7\n",
                ),
                Line::Bestmove(Root::A) => MOVE_A,
                Line::Bestmove(Root::B) => MOVE_B,
                Line::Ready => READY_OK,
                Line::PositionRejected => REJECT_POSITION,
                Line::InvalidPosition => {
                    Response("info string position rejected: invalid FEN: no king\n")
                }
                Line::GoRejected => REJECT_GO,
            },
        }
    }

    #[test]
    #[expect(
        clippy::print_stderr,
        reason = "report the size of the bounded model run"
    )]
    fn model_histories_obey_session_rules() {
        let moves = [forced_move(A), forced_move(B)];
        let events = |history: &[Observation]| {
            std::iter::once(POSITION_A)
                .chain(
                    history
                        .iter()
                        .map(|&observation| event(observation, &moves)),
                )
                .collect::<Vec<_>>()
        };
        let correct = generate(Engine::Correct);
        assert!(!correct.is_empty());
        eprintln!("Correct engine: {} histories", correct.len());
        for history in &correct {
            accepts(&events(history));
        }
        for engine in [
            Engine::LateRejection,
            Engine::LateReady,
            Engine::IdleStopRepeats,
            Engine::QuitDropsMove,
            Engine::ReadyReleasesMove,
            Engine::NoStopBarrier,
            Engine::ReadyAfterQuit,
        ] {
            let generated = generate(engine);
            // Some schedules never exercise the fault or are observationally
            // identical to a permitted handling order after finite BestMove.
            let violations = generated.difference(&correct);
            assert!(
                violations.clone().next().is_some(),
                "{engine:?} emitted no bad history"
            );
            eprintln!("{engine:?}: {} bad histories", violations.clone().count());
            for history in violations {
                assert!(
                    check(&events(history)).is_err(),
                    "{engine:?} accepted: {history:?}"
                );
            }
        }
    }
}
