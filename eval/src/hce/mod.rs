//! Hand-crafted evaluation.

mod params;
mod phase;
mod psqt;
mod taper;

use engine::{Board, Color, Piece, PieceType};

use params::MATERIAL;
use phase::{phase, tapered};
use psqt::{piece_square, psqt};
pub use taper::Taper;

/// Static evaluation from White's point of view, in centipawns.
#[inline]
pub(crate) fn evaluate_white(board: &Board) -> i32 {
    let mut total = Taper::ZERO;
    for color in Color::ALL {
        for t in PieceType::ALL {
            let piece = Piece::new(color, t);
            for sq in board.piece_bb(color, t) {
                total += psqt(piece, sq);
            }
        }
    }
    tapered(total, phase(board))
}

/// The evaluation split by term.
#[must_use]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Trace {
    material: [Taper; Color::COUNT],
    piece_square: [Taper; Color::COUNT],
    phase: i32,
    total: i32,
}

impl Trace {
    /// Material of `color`, from its own point of view.
    #[inline]
    pub const fn material(&self, color: Color) -> Taper {
        self.material[color.index()]
    }

    /// Piece-square value of `color`, from its own point of view.
    #[inline]
    pub const fn piece_square(&self, color: Color) -> Taper {
        self.piece_square[color.index()]
    }

    /// Game phase, from 0 (pure endgame) to 24 (pure middlegame).
    #[inline]
    #[must_use]
    pub const fn phase(&self) -> i32 {
        self.phase
    }

    /// Final score after tapering, from White's point of view.
    #[inline]
    #[must_use]
    pub const fn total(&self) -> i32 {
        self.total
    }
}

/// Evaluates `board` term by term. Slow: a second path, without the combined
/// table. In debug, checks that it agrees with the fast path.
pub fn trace(board: &Board) -> Trace {
    let mut material = [Taper::ZERO; Color::COUNT];
    let mut piece_squares = [Taper::ZERO; Color::COUNT];
    for color in Color::ALL {
        for t in PieceType::ALL {
            for sq in board.piece_bb(color, t) {
                material[color.index()] += MATERIAL[t.index()];
                piece_squares[color.index()] += piece_square(t, sq.relative_to(color));
            }
        }
    }
    let white = material[Color::White.index()] + piece_squares[Color::White.index()]
        - material[Color::Black.index()]
        - piece_squares[Color::Black.index()];
    let phase = phase(board);
    let total = tapered(white, phase);
    debug_assert_eq!(
        total,
        evaluate_white(board),
        "trace and evaluation disagree"
    );
    Trace {
        material,
        piece_square: piece_squares,
        phase,
        total,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Score;

    fn board(fen: &str) -> Board {
        Board::from_fen(fen).unwrap_or_else(|e| panic!("{fen}: {e}"))
    }

    const FENS: [&str; 7] = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "rnbqkbnr/pppppp2/6qq/8/8/6QQ/PPPPPP2/RNBQKBNR w KQkq - 0 1",
        "r1bqkbnr/pppp1ppp/2n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3",
        "8/pp3k2/8/3P4/8/8/PP3K2/8 b - - 0 40",
        "rnbqkbnr/pppppppp/8/8/8/8/8/4K3 w kq - 0 1",
        "k7/8/8/8/8/8/1QQQ3K/1QQQQQQ1 w - - 0 1",
    ];

    #[test]
    fn startpos_is_zero() {
        assert_eq!(evaluate_white(&Board::startpos()), 0);
    }

    #[test]
    fn missing_knight_costs_a_knight() {
        let score = evaluate_white(&board(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/R1BQKBNR w KQkq - 0 1",
        ));
        assert!((-350..=-270).contains(&score), "{score}");
    }

    #[test]
    fn both_paths_agree() {
        for fen in FENS {
            let b = board(fen);
            assert_eq!(trace(&b).total(), evaluate_white(&b), "{fen}");
        }
    }

    #[test]
    fn mirror_negates_the_score() {
        for fen in FENS {
            let b = board(fen);
            assert_eq!(evaluate_white(&b.mirror()), -evaluate_white(&b), "{fen}");
        }
    }

    #[test]
    fn trace_terms() {
        let t = trace(&Board::startpos());
        assert_eq!(t.phase(), 24);
        assert_eq!(t.total(), 0);
        assert_eq!(t.material(Color::White), t.material(Color::Black));
        assert_eq!(t.piece_square(Color::White), t.piece_square(Color::Black));
        let start = [8, 2, 2, 2, 1, 1];
        let mut material = Taper::ZERO;
        for t in PieceType::ALL {
            material += MATERIAL[t.index()] * start[t.index()];
        }
        assert_eq!(t.material(Color::White), material);
    }

    #[test]
    fn nine_queens_stay_within_eval_max() {
        let score = evaluate_white(&board(FENS[6]));
        assert!(score > 0 && score <= Score::EVAL_MAX.raw(), "{score}");
    }
}
