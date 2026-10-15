use engine::{Board, Color};

use crate::Score;
use crate::hce;

/// The evaluation entry point for the search. Each search thread owns one.
/// Empty for now: it will hold the NNUE accumulators (phase 8).
#[non_exhaustive]
#[derive(Default, Debug)]
pub struct Evaluator {}

impl Evaluator {
    /// A new evaluator.
    #[must_use]
    pub fn new() -> Evaluator {
        Evaluator {}
    }

    /// Static evaluation of `board`, from the side to move's point of view,
    /// within `±EVAL_MAX`. Valid in any legal position, check included.
    #[inline]
    pub fn evaluate(&mut self, board: &Board) -> Score {
        let white = hce::evaluate_white(board);
        let v = match board.side_to_move() {
            Color::White => white,
            Color::Black => -white,
        };
        let max = Score::EVAL_MAX.raw();
        Score::new(v.clamp(-max, max))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startpos_is_zero() {
        let mut e = Evaluator::new();
        assert_eq!(e.evaluate(&Board::startpos()), Score::DRAW);
        let black = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1");
        assert_eq!(e.evaluate(&black.unwrap()), Score::DRAW);
    }

    #[test]
    fn black_to_move_negates() {
        let mut e = Evaluator::new();
        // White is a knight down.
        for fen in [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/R1BQKBNR w KQkq - 0 1",
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/R1BQKBNR b KQkq - 0 1",
        ] {
            let b = Board::from_fen(fen).unwrap();
            let white = hce::evaluate_white(&b);
            assert!(white < 0, "{fen}");
            let expected = match b.side_to_move() {
                Color::White => white,
                Color::Black => -white,
            };
            assert_eq!(e.evaluate(&b).raw(), expected, "{fen}");
        }
    }

    #[test]
    fn is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Evaluator>();
    }
}
