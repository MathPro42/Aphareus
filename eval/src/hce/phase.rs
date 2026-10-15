use engine::{Board, PieceType};

use super::params::{PHASE_MAX, PHASE_WEIGHTS};
use super::taper::Taper;

/// Game phase from the material of both sides: `PHASE_MAX` is a pure
/// middlegame, 0 a pure endgame. Capped at `PHASE_MAX`.
#[inline]
pub(crate) fn phase(board: &Board) -> i32 {
    let mut p = 0;
    for t in PieceType::ALL {
        p += PHASE_WEIGHTS[t.index()] * board.pieces(t).count() as i32;
    }
    p.min(PHASE_MAX)
}

/// Blends the middlegame and endgame values of `value` by `phase`.
/// Truncating division keeps tapered(-v) == -tapered(v).
#[inline]
#[must_use]
pub(crate) const fn tapered(value: Taper, phase: i32) -> i32 {
    debug_assert!(phase >= 0 && phase <= PHASE_MAX, "phase out of range");
    (value.mg() * phase + value.eg() * (PHASE_MAX - phase)) / PHASE_MAX
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hce::taper::s;

    fn phase_of(fen: &str) -> i32 {
        phase(&Board::from_fen(fen).unwrap())
    }

    #[test]
    fn phase_bounds() {
        assert_eq!(phase(&Board::startpos()), PHASE_MAX);
        assert_eq!(phase_of("4k3/8/8/8/8/8/8/4K3 w - - 0 1"), 0);
        assert_eq!(phase_of("4k3/pppppppp/8/8/8/8/PPPPPPPP/4K3 w - - 0 1"), 0);
    }

    #[test]
    fn phase_weights() {
        // Q = 4, R = 2, B = 1, N = 1.
        assert_eq!(phase_of("4k3/8/8/8/8/8/8/RNBQK3 w - - 0 1"), 8);
        // Both sides count.
        assert_eq!(phase_of("3qk3/8/8/8/8/8/8/3QK3 w - - 0 1"), 8);
    }

    #[test]
    fn phase_is_capped() {
        // Starting material plus two promoted queens per side: 40 before the cap.
        let fen = "rnbqkbnr/pppppp2/6qq/8/8/6QQ/PPPPPP2/RNBQKBNR w KQkq - 0 1";
        assert_eq!(phase_of(fen), PHASE_MAX);
    }

    #[test]
    fn tapered_examples() {
        assert_eq!(tapered(s(100, 0), 24), 100);
        assert_eq!(tapered(s(100, 0), 0), 0);
        assert_eq!(tapered(s(0, 100), 12), 50);
        assert_eq!(tapered(s(0, 100), 24), 0);
        assert_eq!(tapered(s(0, 100), 0), 100);
    }

    #[test]
    fn tapered_is_antisymmetric() {
        for p in 0..=PHASE_MAX {
            for mg in -300..=300 {
                for eg in -300..=300 {
                    let v = s(mg, eg);
                    let t = tapered(v, p);
                    assert_eq!(tapered(-v, p), -t, "mg = {mg}, eg = {eg}, phase = {p}");
                    assert!(
                        mg.min(eg) <= t && t <= mg.max(eg),
                        "mg = {mg}, eg = {eg}, phase = {p}"
                    );
                }
            }
        }
    }
}
