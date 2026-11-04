mod common;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use common::{position, search, search_with};
use engine::movegen::legal_moves;
use engine::{Board, Move, from_uci, to_uci};
use search::{IterationInfo, Limits, RecordingReporter, Reporter, SearchPool};

const STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
/// Mate in 1 on the back rank: Ra8.
const BACK_RANK: &str = "6k1/5ppp/8/8/8/8/8/R5K1 w - - 0 1";
const PERFT_STANDARD: &str = include_str!("../../engine/tests/data/perft_standard.epd");
const PERFT_960: &str = include_str!("../../engine/tests/data/perft_960.epd");

/// Keeps the node count of each iteration.
#[derive(Debug, Default, PartialEq, Eq)]
struct NodeLog {
    iterations: Vec<(u32, u64, Vec<Move>)>,
    best_moves: Vec<Move>,
}

impl Reporter for NodeLog {
    fn iteration(&mut self, info: &IterationInfo<'_>) {
        self.iterations
            .push((info.depth, info.nodes, info.pv.to_vec()));
    }

    fn best_move(&mut self, mv: Move) {
        self.best_moves.push(mv);
    }
}

fn nodes(n: u64) -> Limits {
    Limits {
        nodes: Some(n),
        ..Limits::default()
    }
}

fn depths(r: &RecordingReporter) -> Vec<u32> {
    r.iterations.iter().map(|it| it.0).collect()
}

/// The FENs of an EPD file: the text before the first `;` of each line.
fn fens(epd: &str) -> impl Iterator<Item = &str> {
    epd.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.split(';').next().unwrap().trim())
}

#[test]
fn depth_gives_one_iteration_per_depth() {
    for d in 1..=4 {
        let r = search(position(STARTPOS, ""), Limits::depth(d));
        assert_eq!(depths(&r), (1..=d).collect::<Vec<_>>());
        assert_eq!(r.best_moves.len(), 1);
    }
}

#[test]
fn depth_zero_still_searches_one_ply() {
    let r = search(position(STARTPOS, ""), Limits::depth(0));
    assert_eq!(depths(&r), [1]);
    assert_eq!(r.best_moves, [r.iterations[0].2[0]]);
}

#[test]
fn node_limit_ends_the_search() {
    let n = 10_000;
    let log = search_with(position(STARTPOS, ""), nodes(n), NodeLog::default());
    assert!(log.iterations.len() >= 2);
    // A complete iteration after the first never reaches the limit.
    for &(depth, count, _) in &log.iterations[1..] {
        assert!(count < n, "depth {depth}: {count}");
    }
    assert_eq!(log.best_moves, [log.iterations.last().unwrap().2[0]]);
}

#[test]
fn node_limit_is_reproducible() {
    let run = || search_with(position(STARTPOS, ""), nodes(10_000), NodeLog::default());
    assert_eq!(run(), run());
}

#[test]
fn node_limit_never_cuts_the_first_iteration() {
    let r = search(position(STARTPOS, ""), nodes(1));
    assert_eq!(depths(&r), [1]);
    assert_eq!(r.best_moves, [r.iterations[0].2[0]]);
}

/// With `go infinite`, no best move before `stop`, even once the search has
/// nothing left to do: a mate found, or the depth reached.
#[test]
fn infinite_waits_for_stop() {
    let mate = Limits {
        infinite: true,
        ..Limits::default()
    };
    let depth_one = Limits {
        infinite: true,
        ..Limits::depth(1)
    };
    for (fen, limits) in [(BACK_RANK, mate), (STARTPOS, depth_one)] {
        let rec = Arc::new(Mutex::new(RecordingReporter::default()));
        let mut pool = SearchPool::new();
        pool.start(position(fen, ""), limits, Box::new(Arc::clone(&rec)));
        thread::sleep(Duration::from_millis(100));
        {
            let r = rec.lock().unwrap();
            assert_eq!(depths(&r), [1], "{fen}");
            assert!(r.best_moves.is_empty(), "{fen}");
        }
        pool.stop();
        pool.wait();
        let r = rec.lock().unwrap();
        assert_eq!(r.best_moves, [r.iterations[0].2[0]], "{fen}");
    }
}

const MOVETIME_MARGIN: Duration = if cfg!(debug_assertions) {
    Duration::from_millis(100)
} else {
    Duration::from_millis(20)
};

#[test]
fn movetime_is_respected() {
    let limits = Limits {
        movetime: Some(100),
        ..Limits::default()
    };
    let rec = Arc::new(Mutex::new(RecordingReporter::default()));
    let mut pool = SearchPool::new();
    let start = Instant::now();
    pool.start(position(STARTPOS, ""), limits, Box::new(Arc::clone(&rec)));
    pool.wait();
    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_millis(100) + MOVETIME_MARGIN,
        "{elapsed:?}"
    );
    assert_eq!(rec.lock().unwrap().best_moves.len(), 1);
}

/// Each line is replayed from the root, every move checked against the
/// legal moves and the UCI notation.
fn check_lines(fen: &str, chess960: bool, depth: u32) {
    let root = Board::from_fen(fen).unwrap();
    let r = search(position(fen, ""), Limits::depth(depth));
    assert!(!r.iterations.is_empty(), "{fen}");
    for (d, _, pv) in &r.iterations {
        assert!(!pv.is_empty(), "{fen} depth {d}");
        let mut board = root.clone();
        for &mv in pv {
            assert!(legal_moves(&board).contains(mv), "{fen} depth {d}");
            let text = to_uci(&board, mv, chess960);
            assert_eq!(from_uci(&board, &text, chess960), Some(mv), "{fen}");
            board = board.make_move(mv);
        }
    }
    let (_, _, pv) = r.iterations.last().unwrap();
    assert_eq!(r.best_moves, [pv[0]], "{fen}");
}

#[test]
fn principal_variations_are_legal() {
    for fen in fens(PERFT_STANDARD) {
        check_lines(fen, false, 2);
    }
    for fen in fens(PERFT_960).take(10) {
        check_lines(fen, true, 3);
    }
}

#[test]
#[ignore = "long: run in release with --ignored"]
fn principal_variations_are_legal_full() {
    for fen in fens(PERFT_STANDARD) {
        check_lines(fen, false, 5);
    }
    for fen in fens(PERFT_960) {
        check_lines(fen, true, 4);
    }
}
