mod common;

use engine::{Board, Color};
use eval::{Evaluator, Score, trace};

/// A position and its mirror get opposite scores from White's point
/// of view, term by term, and the same score from the side to move's point
/// of view. Every score stays within `EVAL_MAX`.
fn check(positions: &[Board]) {
    let mut e = Evaluator::new();
    for board in positions {
        let mirror = board.mirror();
        assert_eq!(e.evaluate(&mirror), e.evaluate(board), "{}", board.to_fen());
        let t = trace(board);
        let m = trace(&mirror);
        assert_eq!(m.total(), -t.total(), "{}", board.to_fen());
        assert_eq!(m.phase(), t.phase(), "{}", board.to_fen());
        for c in Color::ALL {
            assert_eq!(m.material(c), t.material(c.flip()), "{}", board.to_fen());
            assert_eq!(
                m.piece_square(c),
                t.piece_square(c.flip()),
                "{}",
                board.to_fen()
            );
        }
        assert!(
            t.total().abs() <= Score::EVAL_MAX.raw(),
            "{}: {}",
            t.total(),
            board.to_fen()
        );
    }
}

#[test]
fn mirror_negates_the_score() {
    check(&common::positions(1));
}

/// The Chess960 suite to depth 2 as well.
#[test]
#[ignore = "long: run in release with --ignored"]
fn mirror_negates_the_score_full() {
    check(&common::positions(2));
}
