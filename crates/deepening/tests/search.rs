use std::{
    error::Error,
    num::NonZeroU8,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

use deepening::{FinishedDepth, Limits, Score, SearchReport, SearchRequest, search};
use rules::{Move, Position};

fn reports_for(fen: &str) -> Result<Vec<SearchReport>, Box<dyn Error>> {
    Ok(reports_with_limits(
        Position::from_fen(fen)?,
        Limits::default(),
    ))
}

fn reports_with_limits(position: Position, limits: Limits) -> Vec<SearchReport> {
    reports_with_setting(position, limits, NonZeroU8::MIN)
}

fn reports_with_setting(
    position: Position,
    limits: Limits,
    requested_variations: NonZeroU8,
) -> Vec<SearchReport> {
    let request = SearchRequest {
        requested_variations,
        position,
        limits,
        interrupt: Arc::new(AtomicBool::new(false)),
        started: Instant::now(),
    };
    let mut reports = Vec::new();
    search(request, &mut |report| reports.push(report));
    reports
}

#[test]
fn first_variation_matches_setting_one_when_root_moves_tie() -> Result<(), Box<dyn Error>> {
    // The search crate uses plain minimax to check the D73 properties of
    // these same positions in tied_root_moves.rs.
    let positions = include_str!("../../../test-data/tied-root-moves.txt");
    for entry in positions
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
        let (setting, fen) = entry.split_once(' ').ok_or("expected setting and FEN")?;
        let position = Position::from_fen(fen)?;
        let setting = setting.parse::<NonZeroU8>()?;
        assert!(
            setting > NonZeroU8::MIN,
            "{fen}: expected several variations"
        );
        let limits = Limits {
            depth: Some(2),
            ..Limits::default()
        };
        let single = reports_with_limits(position, limits.clone());
        let several = reports_with_setting(position, limits, setting);
        let first = first_variation_at_depth_two(&single, 1)?;
        let count = usize::from(setting.get()).min(position.legal_moves().len());
        let other = first_variation_at_depth_two(&several, count)?;
        assert!(
            first == other,
            "{fen} at depth 2, setting {setting}: first variation differs from setting 1"
        );
    }
    Ok(())
}

fn first_variation_at_depth_two(
    reports: &[SearchReport],
    count: usize,
) -> Result<&(Score, Vec<Move>), String> {
    let [
        SearchReport::FinishedDepth(previous),
        SearchReport::FinishedDepth(depth),
        SearchReport::BestMove(Some(best_move)),
    ] = reports
    else {
        return Err("expected two finished depths and a best move".to_owned());
    };
    assert_eq!(previous.depth, 1);
    assert_eq!(depth.depth, 2);
    assert_eq!(depth.variations.len(), count);
    let first = depth.variations.first().ok_or("missing first variation")?;
    assert_eq!(first.1.first(), Some(best_move));
    Ok(first)
}

#[test]
fn reports_terminal_positions_without_searching_a_depth() -> Result<(), Box<dyn Error>> {
    // Positions and scores from expectations/uci/terminal-positions.uci.
    for (fen, score) in [
        (
            "7k/6Q1/5K2/8/8/8/8/8 b - - 0 1",
            Score::Mate {
                plies: 0,
                winning: false,
            },
        ),
        ("7k/5K2/6Q1/8/8/8/8/8 b - - 0 1", Score::Centipawns(0)),
    ] {
        let reports = reports_for(fen)?;
        // A terminal root has no legal first move for a principal variation.
        assert!(
            matches!(
                reports.as_slice(),
                [
                    SearchReport::FinishedDepth(FinishedDepth {
                        depth: 0,
                        terminal_score: Some(actual),
                        variations,
                        ..
                    }),
                    SearchReport::BestMove(None),
                ] if *actual == score && variations.is_empty()
            ),
            "{fen}: {reports:?}"
        );
    }
    Ok(())
}

#[test]
fn reports_the_finished_depth_before_the_best_move() -> Result<(), Box<dyn Error>> {
    // The node bounds are from expectations/uci-depth-and-nodes/README.md.
    let reports = reports_for("7k/8/5K1Q/8/8/8/8/8 b - - 0 1")?;
    let (depths, best_move) = forced_move_reports(&reports)?;
    assert_eq!(depths.len(), 4);
    let report = depths.last().ok_or("missing last finished depth")?;
    let [(score, moves)] = report.variations.as_slice() else {
        return Err("expected one variation".into());
    };
    assert_eq!(
        *score,
        Score::Mate {
            plies: 2,
            winning: false
        }
    );
    assert_eq!(format!("{}{}", best_move.from(), best_move.to()), "h8g8");
    assert_eq!(moves.first(), Some(best_move));
    Ok(())
}

#[test]
fn interrupt_before_the_root_of_a_terminal_position() -> Result<(), Box<dyn std::error::Error>> {
    assert_interrupted_terminal_reports(
        "7k/6Q1/5K2/8/8/8/8/8 b - - 0 1",
        Score::Mate {
            plies: 0,
            winning: false,
        },
    )
}

#[test]
fn interrupt_before_the_root_of_a_stalemate() -> Result<(), Box<dyn Error>> {
    assert_interrupted_terminal_reports("7k/5K2/6Q1/8/8/8/8/8 b - - 0 1", Score::Centipawns(0))
}

fn assert_interrupted_terminal_reports(fen: &str, score: Score) -> Result<(), Box<dyn Error>> {
    // Positions and scores from expectations/uci/terminal-positions.uci.
    let request = SearchRequest {
        requested_variations: std::num::NonZeroU8::MIN,
        position: Position::from_fen(fen)?,
        limits: Limits::default(),
        interrupt: Arc::new(AtomicBool::new(true)),
        started: Instant::now(),
    };
    let mut reports = Vec::new();
    search(request, &mut |report| reports.push(report));
    assert_eq!(
        reports,
        [
            SearchReport::FinishedDepth(FinishedDepth {
                depth: 0,
                nodes: 0,
                elapsed: std::time::Duration::ZERO,
                terminal_score: Some(score),
                variations: Vec::new(),
            }),
            SearchReport::BestMove(None),
        ]
    );
    Ok(())
}

#[test]
fn infinite_search_reaches_past_the_default_depth() -> Result<(), Box<dyn Error>> {
    let interrupt = Arc::new(AtomicBool::new(false));
    let request = SearchRequest {
        requested_variations: std::num::NonZeroU8::MIN,
        position: Position::from_fen("7k/8/5K1Q/8/8/8/8/8 b - - 0 1")?,
        limits: Limits {
            infinite: true,
            ..Limits::default()
        },
        interrupt: Arc::clone(&interrupt),
        started: Instant::now(),
    };
    let mut depths = Vec::new();
    search(request, &mut |report| {
        if let SearchReport::FinishedDepth(report) = report {
            assert!(
                report.depth <= 5,
                "reported depth after interrupt: {report:?}"
            );
            depths.push(report.depth);
            if report.depth == 5 {
                interrupt.store(true, Ordering::Relaxed);
            }
        }
    });
    assert_eq!(depths, [1, 2, 3, 4, 5]);
    Ok(())
}

#[test]
fn explicit_limits_can_search_beyond_four_plies() -> Result<(), Box<dyn Error>> {
    let position = Position::from_fen("7k/8/5K1Q/8/8/8/8/8 b - - 0 1")?;
    let reports = reports_with_limits(
        position,
        Limits {
            depth: Some(5),
            ..Limits::default()
        },
    );
    let (depths, _) = forced_move_reports(&reports)?;
    assert_eq!(depths.len(), 5);

    let reports = reports_with_limits(
        position,
        Limits {
            nodes: Some(3138),
            ..Limits::default()
        },
    );
    let (depths, _) = forced_move_reports(&reports)?;
    assert!(depths.len() >= 5);
    for depth in depths {
        assert!(depth.nodes <= 3138);
    }
    Ok(())
}

fn forced_move_reports(
    reports: &[SearchReport],
) -> Result<(Vec<&FinishedDepth>, &rules::Move), Box<dyn Error>> {
    let Some((SearchReport::BestMove(Some(best_move)), depths)) = reports.split_last() else {
        return Err(format!("unexpected reports: {reports:?}").into());
    };
    assert_eq!(format!("{}{}", best_move.from(), best_move.to()), "h8g8");

    // Node totals from expectations/uci-depth-and-nodes/README.md.
    let bounds = [1..=1, 23..=23, 24..=75, 25..=857, 26..=3138];
    let mut previous_nodes = 0;
    let mut finished = Vec::new();
    for (index, report) in depths.iter().enumerate() {
        let SearchReport::FinishedDepth(report) = report else {
            return Err(format!("unexpected report: {report:?}").into());
        };
        assert_eq!(usize::from(report.depth), index + 1);
        assert!(report.nodes > previous_nodes, "{report:?}");
        if let Some(bound) = bounds.get(index) {
            assert!(bound.contains(&report.nodes), "{report:?}");
        }
        previous_nodes = report.nodes;
        finished.push(report);
    }
    Ok((finished, best_move))
}

#[test]
fn zero_limits_return_the_first_legal_move_or_the_terminal_score() -> Result<(), Box<dyn Error>> {
    let position = Position::from_fen("7k/8/5K1Q/8/8/8/8/8 b - - 0 1")?;
    let terminal = Position::from_fen("7k/6Q1/5K2/8/8/8/8/8 b - - 0 1")?;
    let movement = position.legal_moves().into_iter().next();
    assert!(movement.is_some());
    for limits in [
        Limits {
            depth: Some(0),
            ..Limits::default()
        },
        Limits {
            nodes: Some(0),
            ..Limits::default()
        },
    ] {
        assert_eq!(
            reports_with_limits(position, limits.clone()),
            [SearchReport::BestMove(movement)]
        );
        assert_eq!(
            reports_with_limits(terminal, limits),
            [
                SearchReport::FinishedDepth(FinishedDepth {
                    depth: 0,
                    nodes: 0,
                    elapsed: std::time::Duration::ZERO,
                    terminal_score: Some(Score::Mate {
                        plies: 0,
                        winning: false
                    }),
                    variations: Vec::new(),
                }),
                SearchReport::BestMove(None),
            ]
        );
    }
    Ok(())
}
