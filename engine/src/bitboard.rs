use std::fmt;
use std::iter::FusedIterator;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

use crate::types::{Color, File, Rank, Square};

/// A set of squares: bit `n` stands for the square of index `n` (a1 = 0, h8 = 63).
#[must_use]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Bitboard(pub u64);

impl Bitboard {
    /// No square.
    pub const EMPTY: Bitboard = Bitboard(0);

    /// All 64 squares.
    pub const FULL: Bitboard = Bitboard(0xFFFF_FFFF_FFFF_FFFF);

    /// One bitboard per file, indexed by `File::index()`.
    pub const FILES: [Bitboard; File::COUNT] = {
        let mut files = [Bitboard::EMPTY; File::COUNT];
        let mut i = 0;
        while i < File::COUNT {
            files[i] = Bitboard(0x0101_0101_0101_0101 << i);
            i += 1;
        }
        files
    };

    /// One bitboard per rank, indexed by `Rank::index()`.
    pub const RANKS: [Bitboard; Rank::COUNT] = {
        let mut ranks = [Bitboard::EMPTY; Rank::COUNT];
        let mut i = 0;
        while i < Rank::COUNT {
            ranks[i] = Bitboard(0xFF << (i * 8));
            i += 1;
        }
        ranks
    };

    /// Dark squares (a1 is dark).
    pub const DARK_SQUARES: Bitboard = Bitboard(0xAA55_AA55_AA55_AA55);

    /// Light squares.
    pub const LIGHT_SQUARES: Bitboard = Bitboard(!Bitboard::DARK_SQUARES.0);

    /// The outer ring of the board.
    pub const EDGES: Bitboard = Bitboard(
        Bitboard::FILES[File::A.index()].0
            | Bitboard::FILES[File::H.index()].0
            | Bitboard::RANKS[Rank::R1.index()].0
            | Bitboard::RANKS[Rank::R8.index()].0,
    );

    const NOT_FILE_A: u64 = !Bitboard::FILES[File::A.index()].0;
    const NOT_FILE_H: u64 = !Bitboard::FILES[File::H.index()].0;

    /// The bitboard holding only `sq`.
    #[inline]
    pub const fn from_square(sq: Square) -> Bitboard {
        Bitboard(1 << sq.index())
    }

    /// All squares of file `f`.
    #[inline]
    pub const fn from_file(f: File) -> Bitboard {
        Bitboard::FILES[f.index()]
    }

    /// All squares of rank `r`.
    #[inline]
    pub const fn from_rank(r: Rank) -> Bitboard {
        Bitboard::RANKS[r.index()]
    }

    /// The squares of `self` that are not in `other`.
    #[inline]
    pub const fn without(self, other: Bitboard) -> Bitboard {
        Bitboard(self.0 & !other.0)
    }

    /// Returns `true` if no square is set.
    #[inline]
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns `true` if `sq` is set.
    #[inline]
    #[must_use]
    pub const fn contains(self, sq: Square) -> bool {
        self.0 & Bitboard::from_square(sq).0 != 0
    }

    /// Number of squares set.
    #[inline]
    #[must_use]
    pub const fn count(self) -> u32 {
        self.0.count_ones()
    }

    /// Returns `true` if at least two squares are set.
    #[inline]
    #[must_use]
    pub const fn more_than_one(self) -> bool {
        self.0 & self.0.wrapping_sub(1) != 0
    }

    /// Returns `true` if exactly one square is set.
    #[inline]
    #[must_use]
    pub const fn is_single(self) -> bool {
        !self.is_empty() && !self.more_than_one()
    }

    /// The lowest square set.
    ///
    /// # Panics
    ///
    /// If the bitboard is empty, in every build: the index would be 64 and
    /// `Square::ALL` bounds-checks it. The debug assertion only gives a
    /// clearer message.
    #[inline]
    #[must_use]
    pub const fn lsb(self) -> Square {
        debug_assert!(!self.is_empty(), "lsb of an empty bitboard");
        Square::ALL[self.0.trailing_zeros() as usize]
    }

    /// The highest square set.
    ///
    /// # Panics
    ///
    /// If the bitboard is empty, in every build (see [`Bitboard::lsb`]).
    #[inline]
    #[must_use]
    pub const fn msb(self) -> Square {
        debug_assert!(!self.is_empty(), "msb of an empty bitboard");
        Square::ALL[63 - self.0.leading_zeros() as usize]
    }

    /// Removes and returns the lowest square set.
    ///
    /// # Panics
    ///
    /// If the bitboard is empty, in every build (see [`Bitboard::lsb`]).
    #[inline]
    pub const fn pop_lsb(&mut self) -> Square {
        let sq = self.lsb();
        self.0 &= self.0 - 1;
        sq
    }

    /// Shifts every square one rank up.
    #[inline]
    pub const fn north(self) -> Bitboard {
        Bitboard(self.0 << 8)
    }

    /// Shifts every square one rank down.
    #[inline]
    pub const fn south(self) -> Bitboard {
        Bitboard(self.0 >> 8)
    }

    /// Shifts every square one file right; the h-file is dropped.
    #[inline]
    pub const fn east(self) -> Bitboard {
        Bitboard((self.0 & Bitboard::NOT_FILE_H) << 1)
    }

    /// Shifts every square one file left; the a-file is dropped.
    #[inline]
    pub const fn west(self) -> Bitboard {
        Bitboard((self.0 & Bitboard::NOT_FILE_A) >> 1)
    }

    /// Shifts every square one step up and right.
    #[inline]
    pub const fn north_east(self) -> Bitboard {
        Bitboard((self.0 & Bitboard::NOT_FILE_H) << 9)
    }

    /// Shifts every square one step up and left.
    #[inline]
    pub const fn north_west(self) -> Bitboard {
        Bitboard((self.0 & Bitboard::NOT_FILE_A) << 7)
    }

    /// Shifts every square one step down and right.
    #[inline]
    pub const fn south_east(self) -> Bitboard {
        Bitboard((self.0 & Bitboard::NOT_FILE_H) >> 7)
    }

    /// Shifts every square one step down and left.
    #[inline]
    pub const fn south_west(self) -> Bitboard {
        Bitboard((self.0 & Bitboard::NOT_FILE_A) >> 9)
    }

    /// Shifts one rank forward, as seen by `color`.
    #[inline]
    pub const fn up(self, color: Color) -> Bitboard {
        match color {
            Color::White => self.north(),
            Color::Black => self.south(),
        }
    }

    /// Shifts one step forward and to the left, as seen by `color`.
    #[inline]
    pub const fn up_left(self, color: Color) -> Bitboard {
        match color {
            Color::White => self.north_west(),
            Color::Black => self.south_east(),
        }
    }

    /// Shifts one step forward and to the right, as seen by `color`.
    #[inline]
    pub const fn up_right(self, color: Color) -> Bitboard {
        match color {
            Color::White => self.north_east(),
            Color::Black => self.south_west(),
        }
    }

    /// Vertical mirror (a1 ↔ a8): reverses the order of the 8 bytes.
    #[inline]
    pub const fn flip_vertical(self) -> Bitboard {
        Bitboard(self.0.swap_bytes())
    }
}

impl BitAnd for Bitboard {
    type Output = Bitboard;
    fn bitand(self, rhs: Bitboard) -> Bitboard {
        Bitboard(self.0 & rhs.0)
    }
}

impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Bitboard) {
        self.0 &= rhs.0;
    }
}

impl BitOr for Bitboard {
    type Output = Bitboard;
    fn bitor(self, rhs: Bitboard) -> Bitboard {
        Bitboard(self.0 | rhs.0)
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Bitboard) {
        self.0 |= rhs.0;
    }
}

impl BitXor for Bitboard {
    type Output = Bitboard;
    fn bitxor(self, rhs: Bitboard) -> Bitboard {
        Bitboard(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for Bitboard {
    fn bitxor_assign(&mut self, rhs: Bitboard) {
        self.0 ^= rhs.0;
    }
}

impl Not for Bitboard {
    type Output = Bitboard;
    fn not(self) -> Bitboard {
        Bitboard(!self.0)
    }
}

/// Iterates over the squares set, in increasing order.
///
/// The bitboard is `Copy`, so `for sq in bb` iterates over a copy and leaves
/// `bb` unchanged. To consume a bitboard in place, use `pop_lsb`.
impl IntoIterator for Bitboard {
    type Item = Square;
    type IntoIter = BitboardIter;

    #[inline]
    fn into_iter(self) -> BitboardIter {
        BitboardIter(self)
    }
}

/// Iterator over the squares of a [`Bitboard`], in increasing order.
#[must_use]
#[derive(Clone, Debug)]
pub struct BitboardIter(Bitboard);

impl Iterator for BitboardIter {
    type Item = Square;

    #[inline]
    fn next(&mut self) -> Option<Square> {
        if self.0.is_empty() {
            None
        } else {
            Some(self.0.pop_lsb())
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.0.count() as usize;
        (n, Some(n))
    }
}

impl ExactSizeIterator for BitboardIter {}

impl FusedIterator for BitboardIter {}

/// Draws an 8 × 8 grid, rank 8 at the top: `X` for a set square, `.` otherwise.
impl fmt::Debug for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in Rank::ALL.into_iter().rev() {
            for file in File::ALL {
                let c = if self.contains(Square::new(file, rank)) {
                    'X'
                } else {
                    '.'
                };
                write!(f, "{c}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Shift = fn(Bitboard) -> Bitboard;

    const DIRECTIONS: [(Shift, i8, i8); 8] = [
        (Bitboard::north, 0, 1),
        (Bitboard::south, 0, -1),
        (Bitboard::east, 1, 0),
        (Bitboard::west, -1, 0),
        (Bitboard::north_east, 1, 1),
        (Bitboard::north_west, -1, 1),
        (Bitboard::south_east, 1, -1),
        (Bitboard::south_west, -1, -1),
    ];

    fn bb(squares: &[Square]) -> Bitboard {
        squares
            .iter()
            .fold(Bitboard::EMPTY, |b, &sq| b | Bitboard::from_square(sq))
    }

    #[test]
    fn from_square() {
        for sq in Square::ALL {
            let b = Bitboard::from_square(sq);
            assert_eq!(b.count(), 1);
            for other in Square::ALL {
                assert_eq!(b.contains(other), other == sq);
            }
        }
    }

    #[test]
    fn masks() {
        for f in File::ALL {
            assert_eq!(Bitboard::from_file(f).count(), 8);
        }
        for r in Rank::ALL {
            assert_eq!(Bitboard::from_rank(r).count(), 8);
        }
        assert_eq!(Bitboard::FULL.count(), 64);
        assert_eq!(Bitboard::DARK_SQUARES.count(), 32);
        assert_eq!(Bitboard::LIGHT_SQUARES.count(), 32);
        assert!(Bitboard::DARK_SQUARES.contains(Square::A1));
        assert!(Bitboard::LIGHT_SQUARES.contains(Square::H1));
        assert_eq!(Bitboard::EDGES.count(), 28);
        assert_eq!(Bitboard::default(), Bitboard::EMPTY);
    }

    #[test]
    fn iteration() {
        let squares = [Square::A1, Square::H1, Square::E4, Square::D5, Square::H8];
        let b = bb(&squares);
        assert_eq!(b.into_iter().len(), b.count() as usize);
        let mut sorted = squares;
        sorted.sort();
        assert_eq!(b.into_iter().collect::<Vec<_>>(), sorted);
        // Iterating over a copy leaves the original unchanged.
        for _ in b {}
        assert_eq!(b.count(), 5);
    }

    #[test]
    fn lsb_msb() {
        let b = bb(&[Square::C3, Square::F6, Square::B7]);
        assert_eq!(b.lsb(), Square::C3);
        assert_eq!(b.msb(), Square::B7);
        assert_eq!(Bitboard::FULL.lsb(), Square::A1);
        assert_eq!(Bitboard::FULL.msb(), Square::H8);
        assert_eq!(Bitboard::from_square(Square::E4).lsb(), Square::E4);
        assert_eq!(Bitboard::from_square(Square::E4).msb(), Square::E4);
    }

    #[test]
    fn pop_lsb_empties() {
        for start in [Bitboard::FULL, Bitboard::DARK_SQUARES, Bitboard::EDGES] {
            let mut b = start;
            for _ in 0..start.count() {
                b.pop_lsb();
            }
            assert!(b.is_empty());
        }
    }

    #[test]
    fn more_than_one() {
        assert!(!Bitboard::EMPTY.more_than_one());
        assert!(!Bitboard::from_square(Square::E4).more_than_one());
        assert!(bb(&[Square::E4, Square::E5]).more_than_one());
        assert!(Bitboard::FULL.more_than_one());

        assert!(!Bitboard::EMPTY.is_single());
        assert!(Bitboard::from_square(Square::H8).is_single());
        assert!(!Bitboard::FULL.is_single());
    }

    #[test]
    fn shifts_drop_edges() {
        assert!(Bitboard::from_file(File::H).east().is_empty());
        assert!(Bitboard::from_file(File::A).west().is_empty());
        assert!(Bitboard::from_rank(Rank::R8).north().is_empty());
        assert!(Bitboard::from_rank(Rank::R1).south().is_empty());
    }

    #[test]
    fn shifts_match_offset() {
        for sq in Square::ALL {
            for (shift, df, dr) in DIRECTIONS {
                let expected = sq
                    .offset(df, dr)
                    .map_or(Bitboard::EMPTY, Bitboard::from_square);
                assert_eq!(
                    shift(Bitboard::from_square(sq)),
                    expected,
                    "{sq} ({df}, {dr})"
                );
            }
        }
    }

    #[test]
    fn relative_shifts() {
        let e4 = Bitboard::from_square(Square::E4);
        assert_eq!(e4.up(Color::White), Bitboard::from_square(Square::E5));
        assert_eq!(e4.up(Color::Black), Bitboard::from_square(Square::E3));
        assert_eq!(e4.up_left(Color::White), Bitboard::from_square(Square::D5));
        assert_eq!(e4.up_left(Color::Black), Bitboard::from_square(Square::F3));
        assert_eq!(e4.up_right(Color::White), Bitboard::from_square(Square::F5));
        assert_eq!(e4.up_right(Color::Black), Bitboard::from_square(Square::D3));
    }

    #[test]
    fn flip_vertical() {
        for b in [
            Bitboard::DARK_SQUARES,
            Bitboard::EDGES,
            bb(&[Square::B2, Square::G7, Square::E4]),
        ] {
            assert_eq!(b.flip_vertical().flip_vertical(), b);
        }
        assert_eq!(
            Bitboard::from_square(Square::A1).flip_vertical(),
            Bitboard::from_square(Square::A8)
        );
    }

    #[test]
    fn debug_grid() {
        let b = bb(&[Square::A1, Square::E4, Square::H8]);
        let expected = "\
.......X
........
........
........
....X...
........
........
X.......
";
        assert_eq!(format!("{b:?}"), expected);
    }
}
