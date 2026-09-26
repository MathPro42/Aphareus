use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

use crate::types::Color;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(u8)]
pub enum CastleSide {
    /// Short castle.
    King = 0,
    /// Long castle.
    Queen = 1,
}

impl CastleSide {
    /// Number of castle sides.
    pub const COUNT: usize = 2;

    /// All castle sides in order.
    pub const ALL: [CastleSide; CastleSide::COUNT] = [CastleSide::King, CastleSide::Queen];

    /// Index for array lookups.
    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct CastlingRights(u8);

impl CastlingRights {
    /// No rights.
    pub const NONE: CastlingRights = CastlingRights(0);

    /// White king side (`K`).
    pub const WHITE_KING_SIDE: CastlingRights = CastlingRights(1);

    /// White queen side (`Q`).
    pub const WHITE_QUEEN_SIDE: CastlingRights = CastlingRights(2);

    /// Black king side (`k`).
    pub const BLACK_KING_SIDE: CastlingRights = CastlingRights(4);

    /// Black queen side (`q`).
    pub const BLACK_QUEEN_SIDE: CastlingRights = CastlingRights(8);

    /// All four rights.
    pub const ALL: CastlingRights = CastlingRights(15);

    /// Number of distinct combinations.
    pub const COUNT: usize = 16;

    /// FEN letters, in bit order.
    const CHARS: [char; 4] = ['K', 'Q', 'k', 'q'];

    /// The single bit for this color and side.
    #[inline]
    const fn bit(color: Color, side: CastleSide) -> u8 {
        1 << (color.index() * 2 + side.index())
    }

    /// Returns `true` if the right is available.
    #[inline]
    pub const fn has(self, color: Color, side: CastleSide) -> bool {
        self.0 & Self::bit(color, side) != 0
    }

    /// Returns these rights with the given one added.
    #[inline]
    pub const fn add(self, color: Color, side: CastleSide) -> CastlingRights {
        CastlingRights(self.0 | Self::bit(color, side))
    }

    /// Returns these rights with the given one removed.
    #[inline]
    pub const fn remove(self, color: Color, side: CastleSide) -> CastlingRights {
        CastlingRights(self.0 & !Self::bit(color, side))
    }

    /// Returns these rights with both of `color`'s removed (the king moved).
    #[inline]
    pub const fn remove_color(self, color: Color) -> CastlingRights {
        CastlingRights(self.0 & !(3 << (color.index() * 2)))
    }

    /// Returns only `color`'s rights.
    #[inline]
    pub const fn for_color(self, color: Color) -> CastlingRights {
        CastlingRights(self.0 & (3 << (color.index() * 2)))
    }

    /// Returns `true` if no right is available.
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Index for array lookups (0..16).
    #[inline]
    pub const fn index(self) -> usize {
        self.0 as usize
    }

    /// Builds rights from their bits, or returns `None` if any high bit is set.
    #[inline]
    pub const fn from_bits(bits: u8) -> Option<CastlingRights> {
        if bits <= 15 {
            Some(CastlingRights(bits))
        } else {
            None
        }
    }

    /// Swaps the White and Black rights.
    #[inline]
    pub const fn flip(self) -> CastlingRights {
        CastlingRights(((self.0 & 3) << 2) | (self.0 >> 2))
    }

    /// Parses the standard FEN castling field: `-`, or a subset of `KQkq`
    /// where each letter appears at most once. Chess960 forms are rejected.
    pub fn parse_standard(s: &str) -> Option<CastlingRights> {
        if s == "-" {
            return Some(CastlingRights::NONE);
        }
        if s.is_empty() {
            return None;
        }
        let mut bits = 0;
        for c in s.chars() {
            let i = Self::CHARS.iter().position(|&x| x == c)?;
            if bits & (1 << i) != 0 {
                return None;
            }
            bits |= 1 << i;
        }
        Some(CastlingRights(bits))
    }
}

impl BitAnd for CastlingRights {
    type Output = CastlingRights;
    fn bitand(self, rhs: CastlingRights) -> CastlingRights {
        CastlingRights(self.0 & rhs.0)
    }
}

impl BitAndAssign for CastlingRights {
    fn bitand_assign(&mut self, rhs: CastlingRights) {
        self.0 &= rhs.0;
    }
}

impl BitOr for CastlingRights {
    type Output = CastlingRights;
    fn bitor(self, rhs: CastlingRights) -> CastlingRights {
        CastlingRights(self.0 | rhs.0)
    }
}

impl BitOrAssign for CastlingRights {
    fn bitor_assign(&mut self, rhs: CastlingRights) {
        self.0 |= rhs.0;
    }
}

impl Not for CastlingRights {
    type Output = CastlingRights;
    /// Complement within the 4 low bits.
    fn not(self) -> CastlingRights {
        CastlingRights(!self.0 & CastlingRights::ALL.0)
    }
}

impl fmt::Display for CastlingRights {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return write!(f, "-");
        }
        for (i, c) in Self::CHARS.into_iter().enumerate() {
            if self.0 & (1 << i) != 0 {
                write!(f, "{c}")?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SINGLES: [CastlingRights; 4] = [
        CastlingRights::WHITE_KING_SIDE,
        CastlingRights::WHITE_QUEEN_SIDE,
        CastlingRights::BLACK_KING_SIDE,
        CastlingRights::BLACK_QUEEN_SIDE,
    ];

    fn all_combinations() -> impl Iterator<Item = CastlingRights> {
        (0..16).map(|b| CastlingRights::from_bits(b).unwrap())
    }

    // Structure

    #[test]
    fn side_in_index() {
        for (i, side) in CastleSide::ALL.into_iter().enumerate() {
            assert_eq!(side.index(), i);
        }
    }

    #[test]
    fn singles_are_distinct_powers_of_two() {
        let mut union = CastlingRights::NONE;
        for (i, r) in SINGLES.into_iter().enumerate() {
            assert!(r.index().is_power_of_two());
            for other in &SINGLES[i + 1..] {
                assert_ne!(r, *other);
            }
            union |= r;
        }
        assert_eq!(union, CastlingRights::ALL);
    }

    #[test]
    fn bounds() {
        assert_eq!(CastlingRights::NONE.index(), 0);
        assert_eq!(CastlingRights::ALL.index(), 15);
        assert_eq!(CastlingRights::default(), CastlingRights::NONE);
    }

    #[test]
    fn layout() {
        let w = Color::White;
        let b = Color::Black;
        assert_eq!(
            CastlingRights::NONE.add(w, CastleSide::King),
            CastlingRights::WHITE_KING_SIDE
        );
        assert_eq!(
            CastlingRights::NONE.add(w, CastleSide::Queen),
            CastlingRights::WHITE_QUEEN_SIDE
        );
        assert_eq!(
            CastlingRights::NONE.add(b, CastleSide::King),
            CastlingRights::BLACK_KING_SIDE
        );
        assert_eq!(
            CastlingRights::NONE.add(b, CastleSide::Queen),
            CastlingRights::BLACK_QUEEN_SIDE
        );
    }

    #[test]
    fn add_remove() {
        for rights in all_combinations() {
            for color in Color::ALL {
                for side in CastleSide::ALL {
                    let added = rights.add(color, side);
                    let removed = rights.remove(color, side);
                    assert!(added.has(color, side));
                    assert!(!removed.has(color, side));
                    for c in Color::ALL {
                        for s in CastleSide::ALL {
                            if (c, s) != (color, side) {
                                assert_eq!(added.has(c, s), rights.has(c, s));
                                assert_eq!(removed.has(c, s), rights.has(c, s));
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn specific_cases() {
        let kq = CastlingRights::BLACK_KING_SIDE | CastlingRights::BLACK_QUEEN_SIDE;
        assert_eq!(CastlingRights::ALL.remove_color(Color::White), kq);
        assert_eq!(CastlingRights::ALL.for_color(Color::Black), kq);
        let mask = !CastlingRights::WHITE_KING_SIDE;
        assert_eq!((CastlingRights::ALL & mask).to_string(), "Qkq");
    }

    #[test]
    fn from_bits() {
        for b in 0..16u8 {
            assert_eq!(
                CastlingRights::from_bits(b).map(|r| r.index()),
                Some(b as usize)
            );
        }
        assert_eq!(CastlingRights::from_bits(16), None);
        assert_eq!(CastlingRights::from_bits(255), None);
    }

    #[test]
    fn not_stays_in_low_bits() {
        assert_eq!(!CastlingRights::NONE, CastlingRights::ALL);
        for rights in all_combinations() {
            assert!((!rights).index() < 16);
        }
    }

    #[test]
    fn flip() {
        assert_eq!(CastlingRights::ALL.flip(), CastlingRights::ALL);
        assert_eq!(
            CastlingRights::WHITE_KING_SIDE.flip(),
            CastlingRights::BLACK_KING_SIDE
        );
        let qk = CastlingRights::WHITE_QUEEN_SIDE | CastlingRights::BLACK_KING_SIDE;
        let kq = CastlingRights::WHITE_KING_SIDE | CastlingRights::BLACK_QUEEN_SIDE;
        assert_eq!(qk.flip(), kq);
        for rights in all_combinations() {
            assert_eq!(rights.flip().flip(), rights);
        }
    }

    #[test]
    fn display() {
        let texts: Vec<String> = all_combinations().map(|r| r.to_string()).collect();
        assert_eq!(
            texts,
            [
                "-", "K", "Q", "KQ", "k", "Kk", "Qk", "KQk", "q", "Kq", "Qq", "KQq", "kq", "Kkq",
                "Qkq", "KQkq",
            ]
        );
    }

    #[test]
    fn parse_round_trip() {
        for rights in all_combinations() {
            assert_eq!(
                CastlingRights::parse_standard(&rights.to_string()),
                Some(rights)
            );
        }
    }

    #[test]
    fn parse_rejects() {
        for s in ["", "KK", "KQkqK", "x", "-K", "K-", "A"] {
            assert_eq!(CastlingRights::parse_standard(s), None, "accepted {s:?}");
        }
    }
}
