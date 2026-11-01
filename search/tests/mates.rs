mod common;

use common::{position, search};
use engine::{Move, to_uci};
use eval::Score;
use search::Limits;

const MATES: &str = include_str!("data/mates.epd");

/// The lines of `mates.epd`: FEN, every key move, mate distance in moves.
fn mates() -> Vec<(&'static str, Vec<&'static str>, u32)> {
    MATES
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let fields: Vec<&str> = line.split(';').map(str::trim).collect();
            let keys = fields[1].strip_prefix("bm ").unwrap();
            let moves = fields[2].strip_prefix("dm ").unwrap();
            (
                fields[0],
                keys.split_whitespace().collect(),
                moves.parse().unwrap(),
            )
        })
        .collect()
}

#[test]
fn mates_stop_the_search_at_their_distance() {
    for (fen, keys, n) in mates() {
        let plies = 2 * n - 1;
        // Two more plies allowed: the mate alone stops the search.
        let r = search(position(fen, ""), Limits::depth(plies + 2));
        let depths: Vec<u32> = r.iterations.iter().map(|it| it.0).collect();
        assert_eq!(depths, (1..=plies).collect::<Vec<_>>(), "{fen}");

        let (_, score, pv) = r.iterations.last().unwrap();
        assert_eq!(*score, Score::mate_in(plies as usize), "{fen}");
        assert_eq!(pv.len(), plies as usize, "{fen}");
        assert_eq!(r.best_moves, [pv[0]], "{fen}");
        let key = to_uci(position(fen, "").board(), pv[0], false);
        assert!(keys.contains(&key.as_str()), "{fen}: {key}");
    }
}

#[test]
fn mated_side_sees_the_distance() {
    for (fen, keys, n) in mates().into_iter().filter(|m| m.2 > 1) {
        let plies = 2 * n - 2;
        let r = search(position(fen, keys[0]), Limits::depth(plies));
        let (depth, score, _) = r.iterations.last().unwrap();
        assert_eq!(*depth, plies, "{fen}");
        assert_eq!(*score, Score::mated_in(plies as usize), "{fen}");
    }
}

#[test]
fn checkmate_at_the_root_reports_no_move() {
    for (fen, keys, _) in mates().into_iter().filter(|m| m.2 == 1) {
        let r = search(position(fen, keys[0]), Limits::depth(3));
        assert!(r.iterations.is_empty(), "{fen}");
        assert_eq!(r.best_moves, [Move::NULL], "{fen}");
    }
}

#[test]
fn stalemate_at_the_root_reports_no_move() {
    let r = search(
        position("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1", ""),
        Limits::depth(3),
    );
    assert!(r.iterations.is_empty());
    assert_eq!(r.best_moves, [Move::NULL]);
}

#[test]
fn stalemate_in_the_tree_is_avoided() {
    // Qxg6 takes the last black piece and stalemates; Kxg6 wins.
    let fen = "7k/5K2/6r1/8/8/8/8/6Q1 w - - 0 1";
    let r = search(position(fen, ""), Limits::depth(2));
    let (_, score, _) = r.iterations.last().unwrap();
    assert!(*score > Score::new(500));
    assert_ne!(
        to_uci(position(fen, "").board(), r.best_moves[0], false),
        "g1g6"
    );
}
