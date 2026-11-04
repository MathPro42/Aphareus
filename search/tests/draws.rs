mod common;

use common::{position, search};
use engine::{Move, to_uci};
use eval::Score;
use search::{Limits, RecordingReporter};

/// Clock at 99: Qf8 mates on the hundredth ply, any other move draws.
const FIFTY_MATE: &str = "7k/8/6K1/8/8/8/8/5Q2 w - - 99 80";
/// Same material, no mate in 1, clock at 99: every move draws.
const FIFTY_NO_MATE: &str = "7k/8/8/5K2/8/8/8/5Q2 w - - 99 80";
/// King and knight against king.
const KNIGHT_ONLY: &str = "8/8/4k3/8/8/3NK3/8/8 w - - 0 1";
/// White is lost but has a perpetual check: Qh5+ Kg8 Qe8+ Kh7.
const PERPETUAL: &str = "7k/q5p1/8/7Q/8/8/rr6/7K b - - 0 1";
/// White is winning and could give the same checks.
const WINNING: &str = "7k/6p1/8/7Q/8/8/8/1R4K1 b - - 0 1";
/// Twice through the cycle: the root (Kh7, Qe8) has been seen before,
/// and Qh5+ Kg8 reaches a third occurrence.
const CYCLE: &str = "h8g8 h5e8 g8h7 e8h5 h7g8 h5e8 g8h7";

fn last_score(r: &RecordingReporter) -> Score {
    r.iterations.last().unwrap().1
}

fn best_uci(fen: &str, moves: &str, r: &RecordingReporter) -> String {
    to_uci(position(fen, moves).board(), r.best_moves[0], false)
}

#[test]
fn mate_on_the_hundredth_ply_is_a_mate() {
    let r = search(position(FIFTY_MATE, ""), Limits::depth(3));
    assert_eq!(last_score(&r), Score::mate_in(1));
    assert_eq!(best_uci(FIFTY_MATE, "", &r), "f1f8");
}

#[test]
fn fifty_moves_without_mate_is_a_draw() {
    let r = search(position(FIFTY_NO_MATE, ""), Limits::depth(2));
    assert_eq!(last_score(&r), Score::DRAW);

    // The clock alone makes the draw.
    let r = search(
        position(&FIFTY_NO_MATE.replace(" 99 ", " 0 "), ""),
        Limits::depth(2),
    );
    assert!(last_score(&r) > Score::new(500));
}

#[test]
fn insufficient_material_is_a_draw() {
    let r = search(position(KNIGHT_ONLY, ""), Limits::depth(2));
    assert_eq!(last_score(&r), Score::DRAW);
}

#[test]
fn drawn_root_still_plays_a_move() {
    let fen = KNIGHT_ONLY.replace(" 0 1", " 100 80");
    let r = search(position(&fen, ""), Limits::depth(2));
    assert_eq!(r.iterations.len(), 2);
    assert_eq!(r.best_moves.len(), 1);
    assert_ne!(r.best_moves[0], Move::NULL);
}

#[test]
fn losing_side_finds_the_perpetual() {
    let r = search(position(PERPETUAL, CYCLE), Limits::depth(3));
    assert_eq!(last_score(&r), Score::DRAW);
    assert_eq!(best_uci(PERPETUAL, CYCLE, &r), "e8h5");
}

#[test]
fn repetition_needs_the_game_history() {
    let r = search(position(PERPETUAL, CYCLE), Limits::depth(2));
    assert_eq!(last_score(&r), Score::DRAW);
    let fen = position(PERPETUAL, CYCLE).board().to_fen();
    let r = search(position(&fen, ""), Limits::depth(2));
    assert!(last_score(&r) < Score::DRAW);
}

#[test]
fn winning_side_does_not_repeat() {
    let r = search(position(WINNING, CYCLE), Limits::depth(3));
    assert!(last_score(&r) > Score::new(500));
    assert_ne!(best_uci(WINNING, CYCLE, &r), "e8h5");
}
