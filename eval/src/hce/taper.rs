use std::ops::{Add, AddAssign, Mul, Neg, Sub};

/// A pair of values: middlegame (`mg`) and endgame (`eg`), blended by the game phase.
/// Read-only outside this crate.
#[must_use]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Taper {
    mg: i32,
    eg: i32,
}

/// Short constructor, for the tables of `params.rs`.
#[inline]
pub(crate) const fn s(mg: i32, eg: i32) -> Taper {
    Taper { mg, eg }
}

impl Taper {
    /// Zero in both phases.
    pub(crate) const ZERO: Taper = s(0, 0);

    /// The middlegame value.
    #[inline]
    #[must_use]
    pub const fn mg(self) -> i32 {
        self.mg
    }

    /// The endgame value.
    #[inline]
    #[must_use]
    pub const fn eg(self) -> i32 {
        self.eg
    }

    #[inline]
    pub(crate) const fn add(self, rhs: Taper) -> Taper {
        s(self.mg + rhs.mg, self.eg + rhs.eg)
    }

    #[inline]
    pub(crate) const fn sub(self, rhs: Taper) -> Taper {
        s(self.mg - rhs.mg, self.eg - rhs.eg)
    }

    #[inline]
    pub(crate) const fn neg(self) -> Taper {
        s(-self.mg, -self.eg)
    }

    #[inline]
    pub(crate) const fn mul(self, k: i32) -> Taper {
        s(self.mg * k, self.eg * k)
    }
}

impl Add for Taper {
    type Output = Taper;
    fn add(self, rhs: Taper) -> Taper {
        Taper::add(self, rhs)
    }
}

impl AddAssign for Taper {
    fn add_assign(&mut self, rhs: Taper) {
        *self = Taper::add(*self, rhs);
    }
}

impl Sub for Taper {
    type Output = Taper;
    fn sub(self, rhs: Taper) -> Taper {
        Taper::sub(self, rhs)
    }
}

impl Neg for Taper {
    type Output = Taper;
    fn neg(self) -> Taper {
        Taper::neg(self)
    }
}

impl Mul<i32> for Taper {
    type Output = Taper;
    fn mul(self, k: i32) -> Taper {
        Taper::mul(self, k)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_and_fields() {
        let v = s(12, -34);
        assert_eq!((v.mg(), v.eg()), (12, -34));
        assert_eq!((Taper::ZERO.mg(), Taper::ZERO.eg()), (0, 0));
    }

    #[test]
    fn operators() {
        let a = s(100, -20);
        let b = s(30, 50);
        assert_eq!(a + b, s(130, 30));
        assert_eq!(a - b, s(70, -70));
        assert_eq!(-a, s(-100, 20));
        assert_eq!(-(-a), a);
        assert_eq!(a * 3, s(300, -60));
        assert_eq!(a * -1, -a);
        assert_eq!(a + Taper::ZERO, a);
        assert_eq!(a - a, Taper::ZERO);
        let mut c = a;
        c += b;
        assert_eq!(c, a + b);
    }

    #[test]
    fn const_methods_match_operators() {
        const A: Taper = s(7, -9);
        const B: Taper = s(-4, 11);
        const SUM: Taper = A.add(B);
        const DIFF: Taper = A.sub(B);
        const NEG: Taper = A.neg();
        const PROD: Taper = A.mul(5);
        assert_eq!(SUM, A + B);
        assert_eq!(DIFF, A - B);
        assert_eq!(NEG, -A);
        assert_eq!(PROD, A * 5);
    }
}
