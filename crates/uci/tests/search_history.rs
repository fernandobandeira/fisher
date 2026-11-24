//! Hand-written histories from fisher-8ie and D11, D20, D24 and D25,
//! read 2026-11-24. The two forced moves come from go-parameters.uci.
//! The finished-depth reports are scripted and do not run a search.

#[path = "support/search_history.rs"]
mod search_history;

use std::time::Duration;

use deepening::{FinishedDepth, Score, SearchReport};
use search_history::{Event, Event::*, check};

const A: &str = "7k/8/5K1Q/8/8/8/8/8 b - - 0 1";
const B: &str = "7k/8/5K2/6Q1/8/8/8/8 b - - 0 1";
const POSITION_A: Event<'_> = Command("position fen 7k/8/5K1Q/8/8/8/8/8 b - - 0 1");
const POSITION_B: Event<'_> = Command("position fen 7k/8/5K2/6Q1/8/8/8/8 b - - 0 1");
const GO: Event<'_> = Command("go");
const INFINITE: Event<'_> = Command("go infinite");
const STOP: Event<'_> = Command("stop");
const QUIT: Event<'_> = Command("quit");
const READY: Event<'_> = Command("isready");
const READY_OK: Event<'_> = Reply("readyok\n");
const MOVE_A: Event<'_> = Reply("bestmove h8g8\n");
const MOVE_B: Event<'_> = Reply("bestmove h8h7\n");
const INFO_A: Event<'_> =
    Reply("info depth 1 multipv 1 score cp -900 nodes 1 nps 1000 time 1 pv h8g8\n");
const REJECT_POSITION: Event<'_> = Reply("info string position rejected: waiting for stop\n");
const REJECT_GO: Event<'_> = Reply("info string go rejected: waiting for stop\n");

#[expect(
    clippy::unwrap_used,
    reason = "the fixed FEN must have a sole legal move"
)]
fn forced_move(fen: &str) -> rules::Move {
    let moves = rules::Position::from_fen(fen).unwrap().legal_moves();
    assert_eq!(moves.len(), 1, "{fen}");
    moves.into_iter().next().unwrap()
}

fn ended(fen: &str) -> Event<'static> {
    Report(SearchReport::Ended(Some(forced_move(fen))))
}

fn finished_depth() -> Event<'static> {
    Report(SearchReport::FinishedDepth(FinishedDepth {
        depth: 1,
        score: Score::Centipawns(-900),
        nodes: 1,
        elapsed: Duration::from_millis(1),
        principal_variation: vec![forced_move(A)],
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
    accepts(&[POSITION_A, GO, ended(A), MOVE_A, STOP, QUIT]);
    accepts(&[POSITION_A, GO, STOP, ended(A), MOVE_A, QUIT]);
    rejects(
        1,
        "duplicate bestmove",
        &[POSITION_A, GO, ended(A), MOVE_A, STOP, MOVE_A, QUIT],
    );
    rejects(1, "bestmove without go", &[MOVE_A, QUIT]);
    rejects(
        1,
        "accepted go without bestmove",
        &[POSITION_A, GO, ended(A), QUIT],
    );
}

#[test]
fn rule_2_infinite_holds_its_move() {
    accepts(&[
        POSITION_A,
        INFINITE,
        ended(A),
        READY,
        READY_OK,
        STOP,
        MOVE_A,
        QUIT,
    ]);
    rejects(
        2,
        "early infinite bestmove",
        &[POSITION_A, INFINITE, ended(A), MOVE_A, STOP, QUIT],
    );

    let terminal = Command("position fen 7k/6Q1/5K2/8/8/8/8/8 b - - 0 1");
    let null = Reply("bestmove 0000\n");
    accepts(&[
        terminal.clone(),
        INFINITE,
        Report(SearchReport::Ended(None)),
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
            Report(SearchReport::Ended(None)),
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
        ended(A),
        MOVE_A,
        QUIT,
    ]);
    rejects(
        3,
        "late search info",
        &[POSITION_A, GO, STOP, ended(A), MOVE_A, INFO_A, QUIT],
    );
    rejects(
        3,
        "missing finished depth",
        &[
            POSITION_A,
            GO,
            finished_depth(),
            STOP,
            ended(A),
            MOVE_A,
            QUIT,
        ],
    );
    rejects(
        3,
        "info for an unfinished depth",
        &[POSITION_A, GO, STOP, INFO_A, ended(A), MOVE_A, QUIT],
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
        score: Score::Centipawns(0),
        nodes: 20,
        elapsed: Duration::from_millis(1),
        principal_variation: pawn_move("e4").into_iter().collect(),
    }));
    let info = Reply("info depth 1 multipv 1 score cp 0 nodes 20 nps 20000 time 1 pv e2e4\n");
    rejects(
        3,
        "stop ignored the last finished depth",
        &[
            GO,
            depth,
            info,
            STOP,
            Report(SearchReport::Ended(pawn_move("e3"))),
            Reply("bestmove e2e3\n"),
            QUIT,
        ],
    );
}

#[test]
fn rule_4_idle_stop_is_silent() {
    accepts(&[STOP, READY, READY_OK, QUIT]);
    rejects(
        4,
        "idle stop wrote output",
        &[
            STOP,
            Reply("info string no search\n"),
            READY,
            READY_OK,
            QUIT,
        ],
    );
}

#[test]
fn rule_5_bestmove_is_legal_in_the_accepted_position() {
    accepts(&[POSITION_B, GO, ended(B), MOVE_B, QUIT]);
    rejects(
        5,
        "illegal bestmove",
        &[POSITION_A, GO, ended(A), MOVE_B, QUIT],
    );
    rejects(
        5,
        "0000 with a legal move",
        &[POSITION_A, GO, ended(A), Reply("bestmove 0000\n"), QUIT],
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
        ended(A),
        MOVE_A,
        QUIT,
    ]);
    rejects(
        6,
        "missing readyok during search",
        &[POSITION_A, INFINITE, READY, STOP, ended(A), MOVE_A, QUIT],
    );
    rejects(
        6,
        "isready ended search before readyok",
        &[
            POSITION_A,
            INFINITE,
            READY,
            ended(A),
            MOVE_A,
            READY_OK,
            QUIT,
        ],
    );
    rejects(
        2,
        "isready released infinite search after readyok",
        &[
            POSITION_A,
            INFINITE,
            READY,
            READY_OK,
            ended(A),
            MOVE_A,
            QUIT,
        ],
    );
    rejects(6, "missing idle readyok", &[READY, QUIT]);
}

#[test]
fn rule_7_earlier_command_replies_precede_readyok() {
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
        ended(A),
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
        ended(A),
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
            ended(A),
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
            ended(A),
            STOP,
            MOVE_A,
            GO,
            ended(A),
            MOVE_A,
            QUIT,
        ]);
        accepts(&[
            POSITION_A,
            INFINITE,
            ended(A),
            command,
            diagnostic,
            STOP,
            MOVE_A,
            GO,
            ended(A),
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
        ended(A),
        MOVE_A,
        QUIT,
    ]);
    accepts(&[
        POSITION_A,
        GO,
        ended(A),
        POSITION_B,
        GO,
        MOVE_A,
        ended(B),
        MOVE_B,
        QUIT,
    ]);
    accepts(&[
        POSITION_A,
        INFINITE,
        STOP,
        POSITION_B,
        GO,
        ended(A),
        MOVE_A,
        ended(B),
        MOVE_B,
        QUIT,
    ]);
    accepts(&[
        POSITION_A,
        INFINITE,
        ended(A),
        STOP,
        POSITION_B,
        GO,
        MOVE_A,
        ended(B),
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
            ended(A),
            STOP,
            MOVE_A,
            GO,
            ended(B),
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
            ended(A),
            STOP,
            MOVE_A,
            ended(A),
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
            ended(A),
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
            ended(A),
            MOVE_A,
            REJECT_GO,
            QUIT,
        ],
    );
}

#[test]
fn rule_9_quit_and_end_of_input_release_the_move_then_exit() {
    for exit in [QUIT, EndOfInput] {
        accepts(&[POSITION_A, INFINITE, ended(A), exit.clone(), MOVE_A]);
        accepts(&[POSITION_A, INFINITE, exit.clone(), ended(A), MOVE_A]);
        accepts(&[POSITION_A, GO, exit.clone(), ended(A), MOVE_A]);
        accepts(&[POSITION_A, GO, ended(A), exit.clone(), MOVE_A]);
        accepts(&[POSITION_A, GO, STOP, exit.clone(), ended(A), MOVE_A]);
        accepts(std::slice::from_ref(&exit));
        rejects(
            9,
            "exit omitted bestmove",
            &[POSITION_A, GO, exit.clone(), ended(A)],
        );
        rejects(
            9,
            "output after exit bestmove",
            &[POSITION_A, INFINITE, ended(A), exit, MOVE_A, READY_OK],
        );
    }
}

#[test]
fn rule_10_replies_are_whole_lines() {
    accepts(&[
        Command("uci"),
        Reply("id name fisher test\n"),
        Reply("id author Fernando Bandeira\n"),
        Reply("uciok\n"),
        READY,
        READY_OK,
        QUIT,
    ]);
    rejects(10, "missing newline", &[READY, Reply("readyok"), QUIT]);
    rejects(
        10,
        "two replies in one line",
        &[
            POSITION_A,
            GO,
            READY,
            ended(A),
            Reply("bestmove h8g8readyok\n"),
            QUIT,
        ],
    );
    rejects(
        10,
        "two replies in one event",
        &[READY, READY, Reply("readyok\nreadyok\n"), QUIT],
    );
}
