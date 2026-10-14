use std::ops::{Add, Neg, Sub};

/// A score in centipawns, from the point of view of a side chosen by the caller.
#[must_use]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Score(i32);

impl Score {
    /// Draw.
    pub const DRAW: Score = Score(0);

    /// Maximum search depth, in plies.
    pub const MAX_PLY: usize = 256;

    /// We mate now: the mate score at distance 0.
    pub const MATE: Score = Score(31_000);

    /// Lowest mate score: from here up, a mate was found.
    pub const MATE_IN_MAX: Score = Score(Score::MATE.0 - Score::MAX_PLY as i32);

    /// Bound of any static evaluation, far below the mate scores.
    pub const EVAL_MAX: Score = Score(20_000);

    /// Bound of the alpha-beta windows, above any real score.
    pub const INFINITE: Score = Score(32_000);

    /// No score (search stack, transposition table).
    pub const NONE: Score = Score(32_001);

    /// Wraps a value within `±INFINITE`.
    #[inline]
    pub const fn new(value: i32) -> Score {
        debug_assert!(value.abs() <= Score::INFINITE.0, "score out of range");
        Score(value)
    }

    /// The value in centipawns.
    #[inline]
    #[must_use]
    pub const fn raw(self) -> i32 {
        self.0
    }

    /// We mate in `ply` plies.
    #[inline]
    pub const fn mate_in(ply: usize) -> Score {
        debug_assert!(ply <= Score::MAX_PLY, "ply out of range");
        Score(Score::MATE.0 - ply as i32)
    }

    /// We are mated in `ply` plies.
    #[inline]
    pub const fn mated_in(ply: usize) -> Score {
        debug_assert!(ply <= Score::MAX_PLY, "ply out of range");
        Score(-Score::MATE.0 + ply as i32)
    }

    /// Returns `true` for a mate found, either way.
    #[inline]
    #[must_use]
    pub const fn is_mate(self) -> bool {
        self.0.abs() >= Score::MATE_IN_MAX.0
    }

    /// Returns `true` if we mate.
    #[inline]
    #[must_use]
    pub const fn is_win(self) -> bool {
        self.0 >= Score::MATE_IN_MAX.0
    }

    /// Returns `true` if we are mated.
    #[inline]
    #[must_use]
    pub const fn is_loss(self) -> bool {
        self.0 <= -Score::MATE_IN_MAX.0
    }

    /// Moves (not plies) to mate, for `score mate N` in UCI: positive if we mate,
    /// negative if we are mated. `None` for an ordinary score.
    #[inline]
    #[must_use]
    pub const fn mate_moves(self) -> Option<i32> {
        debug_assert!(self.0.abs() <= Score::MATE.0, "not a real score");
        if !self.is_mate() {
            return None;
        }
        if self.0 > 0 {
            Some((Score::MATE.0 - self.0 + 1) / 2)
        } else {
            Some(-(Score::MATE.0 + self.0) / 2)
        }
    }

    /// Before a transposition table write at `ply`: a mate score is changed from
    /// a distance to the root into a distance to this node. Other scores are unchanged.
    /// `NONE` and `INFINITE` are not scores: store `NONE` without this call.
    #[inline]
    pub const fn to_tt(self, ply: usize) -> Score {
        debug_assert!(ply <= Score::MAX_PLY, "ply out of range");
        debug_assert!(self.0.abs() <= Score::MATE.0, "not a real score");
        if self.is_win() {
            Score(self.0 + ply as i32)
        } else if self.is_loss() {
            Score(self.0 - ply as i32)
        } else {
            self
        }
    }

    /// After a transposition table read at `ply`: the reverse of [`Score::to_tt`].
    /// `NONE` comes back unchanged.
    #[inline]
    pub const fn from_tt(self, ply: usize) -> Score {
        debug_assert!(ply <= Score::MAX_PLY, "ply out of range");
        if self.0 == Score::NONE.0 {
            self
        } else if self.is_win() {
            Score(self.0 - ply as i32)
        } else if self.is_loss() {
            Score(self.0 + ply as i32)
        } else {
            self
        }
    }

    /// The value on 16 bits, for the transposition table.
    #[inline]
    #[must_use]
    pub const fn to_i16(self) -> i16 {
        debug_assert!(
            self.0 >= i16::MIN as i32 && self.0 <= i16::MAX as i32,
            "score does not fit in an i16"
        );
        self.0 as i16
    }

    /// Reads a value stored by [`Score::to_i16`].
    #[inline]
    pub const fn from_i16(value: i16) -> Score {
        Score(value as i32)
    }
}

impl Neg for Score {
    type Output = Score;
    fn neg(self) -> Score {
        Score(-self.0)
    }
}

impl Add for Score {
    type Output = Score;
    fn add(self, rhs: Score) -> Score {
        Score(self.0 + rhs.0)
    }
}

impl Sub for Score {
    type Output = Score;
    fn sub(self, rhs: Score) -> Score {
        Score(self.0 - rhs.0)
    }
}

impl Add<i32> for Score {
    type Output = Score;
    fn add(self, rhs: i32) -> Score {
        Score(self.0 + rhs)
    }
}

impl Sub<i32> for Score {
    type Output = Score;
    fn sub(self, rhs: i32) -> Score {
        Score(self.0 - rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every mate score, both ways.
    fn mate_scores() -> impl Iterator<Item = Score> {
        (0..=Score::MAX_PLY).flat_map(|n| [Score::mate_in(n), Score::mated_in(n)])
    }

    /// Ordinary scores, up to the edges of the mate zone.
    fn ordinary_scores() -> [Score; 9] {
        [
            Score::DRAW,
            Score::new(1),
            Score::new(-1),
            Score::new(150),
            Score::new(-150),
            Score::EVAL_MAX,
            -Score::EVAL_MAX,
            Score::MATE_IN_MAX - 1,
            -Score::MATE_IN_MAX + 1,
        ]
    }

    #[test]
    fn constant_order() {
        assert_eq!(Score::MATE_IN_MAX.raw(), 30_744);
        assert!(Score::EVAL_MAX < Score::MATE_IN_MAX);
        assert!(Score::MATE_IN_MAX <= Score::MATE);
        assert!(Score::MATE < Score::INFINITE);
        assert!(Score::INFINITE < Score::NONE);
        assert!(Score::NONE.raw() <= i32::from(i16::MAX));
    }

    #[test]
    fn new_and_raw() {
        assert_eq!(Score::new(-123).raw(), -123);
        assert_eq!(Score::new(0), Score::DRAW);
    }

    #[test]
    fn operators() {
        let a = Score::new(100);
        let b = Score::new(30);
        assert_eq!(a + b, Score::new(130));
        assert_eq!(a - b, Score::new(70));
        assert_eq!(a + 5, Score::new(105));
        assert_eq!(a - 5, Score::new(95));
        assert_eq!(-a, Score::new(-100));
        assert_eq!(-(-a), a);
        assert!(-a < Score::DRAW && Score::DRAW < a);
    }

    #[test]
    fn negated_mate_is_mated() {
        for n in 0..=Score::MAX_PLY {
            assert_eq!(-Score::mate_in(n), Score::mated_in(n), "n = {n}");
        }
    }

    #[test]
    fn mate_predicates() {
        for n in 0..=Score::MAX_PLY {
            let win = Score::mate_in(n);
            assert!(win.is_mate() && win.is_win() && !win.is_loss(), "n = {n}");
            let loss = Score::mated_in(n);
            assert!(
                loss.is_mate() && loss.is_loss() && !loss.is_win(),
                "n = {n}"
            );
        }
        for s in ordinary_scores() {
            assert!(!s.is_mate() && !s.is_win() && !s.is_loss(), "{s:?}");
        }
    }

    #[test]
    fn mate_moves() {
        assert_eq!(Score::mate_in(1).mate_moves(), Some(1));
        assert_eq!(Score::mate_in(2).mate_moves(), Some(1));
        assert_eq!(Score::mate_in(3).mate_moves(), Some(2));
        assert_eq!(Score::mated_in(0).mate_moves(), Some(0));
        assert_eq!(Score::mated_in(2).mate_moves(), Some(-1));
        assert_eq!(Score::mated_in(4).mate_moves(), Some(-2));
        for s in ordinary_scores() {
            assert_eq!(s.mate_moves(), None, "{s:?}");
        }
    }

    #[test]
    fn tt_round_trip() {
        for ply in 0..=Score::MAX_PLY {
            for s in mate_scores().chain(ordinary_scores()) {
                assert_eq!(s.to_tt(ply).from_tt(ply), s, "{s:?} at ply {ply}");
            }
        }
    }

    #[test]
    fn tt_adjusts_to_the_node() {
        // Mate found at ply 10, 5 plies below the node; read back at ply 4.
        let stored = Score::mate_in(15).to_tt(10);
        assert_eq!(stored, Score::mate_in(5));
        assert_eq!(stored.from_tt(4), Score::mate_in(9));

        let stored = Score::mated_in(16).to_tt(10);
        assert_eq!(stored, Score::mated_in(6));
        assert_eq!(stored.from_tt(4), Score::mated_in(10));

        // A mate at or below the node keeps its distance from the node.
        for n in 0..=Score::MAX_PLY {
            for ply in 0..=n {
                assert_eq!(Score::mate_in(n).to_tt(ply), Score::mate_in(n - ply));
                assert_eq!(Score::mated_in(n).to_tt(ply), Score::mated_in(n - ply));
            }
        }
    }

    #[test]
    fn tt_keeps_score_kind() {
        for ply in 0..=Score::MAX_PLY {
            for s in ordinary_scores() {
                assert_eq!(s.to_tt(ply), s, "{s:?} at ply {ply}");
                assert_eq!(s.from_tt(ply), s, "{s:?} at ply {ply}");
            }
            for n in 0..=Score::MAX_PLY {
                let win = Score::mate_in(n).to_tt(ply);
                assert!(
                    win.is_win() && win < Score::INFINITE,
                    "n = {n}, ply = {ply}"
                );
                let loss = Score::mated_in(n).to_tt(ply);
                assert!(
                    loss.is_loss() && loss > -Score::INFINITE,
                    "n = {n}, ply = {ply}"
                );
            }
        }
    }

    #[test]
    fn tt_keeps_none() {
        for ply in 0..=Score::MAX_PLY {
            assert_eq!(Score::NONE.from_tt(ply), Score::NONE, "ply {ply}");
        }
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "not a real score")]
    fn none_to_tt_panics() {
        let _ = Score::NONE.to_tt(5);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "not a real score")]
    fn infinite_mate_moves_panics() {
        let _ = Score::INFINITE.mate_moves();
    }

    #[test]
    fn i16_round_trip() {
        let constants = [
            Score::DRAW,
            Score::EVAL_MAX,
            Score::MATE_IN_MAX,
            Score::MATE,
            Score::INFINITE,
            Score::NONE,
        ];
        for s in constants {
            assert_eq!(Score::from_i16(s.to_i16()), s, "{s:?}");
            assert_eq!(Score::from_i16((-s).to_i16()), -s, "{s:?}");
        }
        for x in i16::MIN..=i16::MAX {
            assert_eq!(Score::from_i16(x).to_i16(), x);
        }
    }

    #[test]
    fn size() {
        assert_eq!(size_of::<Score>(), 4);
    }
}
