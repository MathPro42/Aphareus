use std::fmt;

use crate::types::Color;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u8)]
pub enum File {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    F = 5,
    G = 6,
    H = 7,
}

impl File {
    /// Number of files.
    pub const COUNT: usize = 8;

    /// All files in order.
    pub const ALL: [File; File::COUNT] = [
        File::A,
        File::B,
        File::C,
        File::D,
        File::E,
        File::F,
        File::G,
        File::H,
    ];

    /// Index for array lookups (0 for a, 7 for h).
    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Horizontal mirror.
    #[inline]
    pub const fn flip(self) -> File {
        File::from_index_bounded(self as u32 ^ 7)
    }

    /// Parses a lowercase file letter (`'a'` to `'h'`).
    /// Any other character returns `None`.
    #[inline]
    pub const fn from_char(c: char) -> Option<File> {
        match c {
            'a'..='h' => Some(File::from_index_bounded((c as u8 - b'a') as u32)),
            _ => None,
        }
    }

    /// Returns the lowercase file letter.
    #[inline]
    pub const fn to_char(self) -> char {
        (b'a' + self as u8) as char
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u8)]
pub enum Rank {
    R1 = 0,
    R2 = 1,
    R3 = 2,
    R4 = 3,
    R5 = 4,
    R6 = 5,
    R7 = 6,
    R8 = 7,
}

impl Rank {
    /// Number of ranks.
    pub const COUNT: usize = 8;

    /// All ranks in order, from 1 to 8.
    pub const ALL: [Rank; Rank::COUNT] = [
        Rank::R1,
        Rank::R2,
        Rank::R3,
        Rank::R4,
        Rank::R5,
        Rank::R6,
        Rank::R7,
        Rank::R8,
    ];

    /// Index for array lookups (0 for rank 1, 7 for rank 8).
    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Vertical mirror.
    #[inline]
    pub const fn flip(self) -> Rank {
        Rank::from_index_bounded(self as u32 ^ 7)
    }

    /// The rank as seen by `color`.
    #[inline]
    pub const fn relative_to(self, color: Color) -> Rank {
        match color {
            Color::White => self,
            Color::Black => self.flip(),
        }
    }

    /// Parses a rank digit (`'1'` to `'8'`).
    /// Any other character returns `None`.
    #[inline]
    pub const fn from_char(c: char) -> Option<Rank> {
        match c {
            '1'..='8' => Some(Rank::from_index_bounded((c as u8 - b'1') as u32)),
            _ => None,
        }
    }

    /// Returns the rank digit.
    #[inline]
    pub const fn to_char(self) -> char {
        (b'1' + self as u8) as char
    }
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

#[rustfmt::skip]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u8)]
pub enum Square {
    A1 = 0,  B1, C1, D1, E1, F1, G1, H1,
    A2 = 8,  B2, C2, D2, E2, F2, G2, H2,
    A3 = 16, B3, C3, D3, E3, F3, G3, H3,
    A4 = 24, B4, C4, D4, E4, F4, G4, H4,
    A5 = 32, B5, C5, D5, E5, F5, G5, H5,
    A6 = 40, B6, C6, D6, E6, F6, G6, H6,
    A7 = 48, B7, C7, D7, E7, F7, G7, H7,
    A8 = 56, B8, C8, D8, E8, F8, G8, H8,
}

impl Square {
    /// Number of squares.
    pub const COUNT: usize = 64;

    /// All squares in index order.
    #[rustfmt::skip]
    pub const ALL: [Square; Square::COUNT] = {
        use Square::*;
        [
            A1, B1, C1, D1, E1, F1, G1, H1,
            A2, B2, C2, D2, E2, F2, G2, H2,
            A3, B3, C3, D3, E3, F3, G3, H3,
            A4, B4, C4, D4, E4, F4, G4, H4,
            A5, B5, C5, D5, E5, F5, G5, H5,
            A6, B6, C6, D6, E6, F6, G6, H6,
            A7, B7, C7, D7, E7, F7, G7, H7,
            A8, B8, C8, D8, E8, F8, G8, H8,
        ]
    };

    /// Builds a square from its file and rank (`index = rank × 8 + file`).
    #[inline]
    pub const fn new(file: File, rank: Rank) -> Square {
        Square::from_index_bounded((rank as u32) << 3 | file as u32)
    }

    /// Index for array lookups.
    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Returns the square with this index, or `None` if it is 64 or more.
    #[inline]
    pub const fn from_index(index: u8) -> Option<Square> {
        if (index as usize) < Square::COUNT {
            Some(Square::from_index_bounded(index as u32))
        } else {
            None
        }
    }

    /// Returns the file of the square.
    #[inline]
    pub const fn file(self) -> File {
        File::from_index_bounded(self as u32 & 7)
    }

    /// Returns the rank of the square.
    #[inline]
    pub const fn rank(self) -> Rank {
        Rank::from_index_bounded(self as u32 >> 3)
    }

    /// Vertical mirror (a1 <-> a8), keeping the file.
    #[inline]
    pub const fn flip_vertical(self) -> Square {
        Square::from_index_bounded(self as u32 ^ 56)
    }

    /// Horizontal mirror (a1 <-> h1), keeping the rank.
    #[inline]
    pub const fn flip_horizontal(self) -> Square {
        Square::from_index_bounded(self as u32 ^ 7)
    }

    /// The square as seen by `color`.
    #[inline]
    pub const fn relative_to(self, color: Color) -> Square {
        match color {
            Color::White => self,
            Color::Black => self.flip_vertical(),
        }
    }

    /// Moves the square by the given deltas, or returns `None` if the result
    /// leaves the board. File and rank are computed separately, so there is
    /// no wrap from one edge to the other.
    #[inline]
    pub const fn offset(self, file_delta: i8, rank_delta: i8) -> Option<Square> {
        // i16 so that no i8 delta can overflow.
        let file = self.file() as i16 + file_delta as i16;
        let rank = self.rank() as i16 + rank_delta as i16;
        if file < 0 || file >= 8 || rank < 0 || rank >= 8 {
            return None;
        }
        Some(Square::new(
            File::from_index_bounded(file as u32),
            Rank::from_index_bounded(rank as u32),
        ))
    }

    /// One rank ahead for `color`, or `None` on the last rank.
    #[inline]
    pub const fn forward(self, color: Color) -> Option<Square> {
        match color {
            Color::White => self.offset(0, 1),
            Color::Black => self.offset(0, -1),
        }
    }

    /// One rank behind for `color`, or `None` on the first rank.
    #[inline]
    pub const fn backward(self, color: Color) -> Option<Square> {
        self.forward(color.flip())
    }

    /// Number of files between the two squares.
    #[inline]
    pub const fn file_distance(self, other: Square) -> u8 {
        (self.file() as u8).abs_diff(other.file() as u8)
    }

    /// Number of ranks between the two squares.
    #[inline]
    pub const fn rank_distance(self, other: Square) -> u8 {
        (self.rank() as u8).abs_diff(other.rank() as u8)
    }

    /// King distance: the larger of the file and rank distances.
    #[inline]
    pub const fn distance(self, other: Square) -> u8 {
        let f = self.file_distance(other);
        let r = self.rank_distance(other);
        if f > r { f } else { r }
    }

    /// Parses a square name such as `"e4"`.
    /// Anything else returns `None`.
    #[inline]
    pub const fn parse(s: &str) -> Option<Square> {
        let bytes = s.as_bytes();
        if bytes.len() != 2 {
            return None;
        }
        match (
            File::from_char(bytes[0] as char),
            Rank::from_char(bytes[1] as char),
        ) {
            (Some(file), Some(rank)) => Some(Square::new(file, rank)),
            _ => None,
        }
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.file(), self.rank())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // File and Rank

    #[test]
    fn file_rank_in_index() {
        for (i, f) in File::ALL.into_iter().enumerate() {
            assert_eq!(f.index(), i);
        }
        for (i, r) in Rank::ALL.into_iter().enumerate() {
            assert_eq!(r.index(), i);
        }
    }

    #[test]
    fn file_rank_char() {
        for f in File::ALL {
            assert_eq!(File::from_char(f.to_char()), Some(f));
        }
        for r in Rank::ALL {
            assert_eq!(Rank::from_char(r.to_char()), Some(r));
        }
    }

    #[test]
    fn file_rank_invalid_chars() {
        for ch in ['i', 'A', '0', '9'] {
            assert_eq!(File::from_char(ch), None, "accepted {ch:?}");
            assert_eq!(Rank::from_char(ch), None, "accepted {ch:?}");
        }
    }

    #[test]
    fn file_rank_flip() {
        for f in File::ALL {
            assert_eq!(f.flip().flip(), f);
        }
        for r in Rank::ALL {
            assert_eq!(r.flip().flip(), r);
        }
        assert_eq!(File::A.flip(), File::H);
        assert_eq!(File::H.flip(), File::A);
        assert_eq!(Rank::R1.flip(), Rank::R8);
        assert_eq!(Rank::R8.flip(), Rank::R1);
    }

    #[test]
    fn rank_relative() {
        for r in Rank::ALL {
            assert_eq!(r.relative_to(Color::White), r);
        }
        assert_eq!(Rank::R1.relative_to(Color::Black), Rank::R8);
        assert_eq!(Rank::R2.relative_to(Color::Black), Rank::R7);
    }

    // Square

    #[test]
    fn anchors() {
        assert_eq!(Square::A1.index(), 0);
        assert_eq!(Square::H1.index(), 7);
        assert_eq!(Square::A8.index(), 56);
        assert_eq!(Square::H8.index(), 63);
        assert_eq!(Square::E4.index(), 28);
    }

    #[test]
    fn square_in_index() {
        for (i, sq) in Square::ALL.into_iter().enumerate() {
            assert_eq!(sq.index(), i);
            assert_eq!(Square::from_index(i as u8), Some(sq));
        }
        assert_eq!(Square::from_index(64), None);
    }

    #[test]
    fn construction() {
        for sq in Square::ALL {
            assert_eq!(Square::new(sq.file(), sq.rank()), sq);
        }
    }

    #[test]
    fn text_round_trip() {
        for sq in Square::ALL {
            assert_eq!(Square::parse(&sq.to_string()), Some(sq));
        }
        assert_eq!(Square::E4.to_string(), "e4");
    }

    #[test]
    fn flips() {
        for sq in Square::ALL {
            assert_eq!(sq.flip_vertical().flip_vertical(), sq);
            assert_eq!(sq.flip_horizontal().flip_horizontal(), sq);
            assert_eq!(sq.flip_vertical().file(), sq.file());
            assert_eq!(sq.flip_horizontal().rank(), sq.rank());
        }
    }

    #[test]
    fn square_relative() {
        for sq in Square::ALL {
            assert_eq!(sq.relative_to(Color::White), sq);
            assert_eq!(sq.relative_to(Color::Black), sq.flip_vertical());
        }
    }

    #[test]
    fn specific_flips() {
        assert_eq!(Square::A1.flip_vertical(), Square::A8);
        assert_eq!(Square::A1.flip_horizontal(), Square::H1);
        assert_eq!(Square::E2.relative_to(Color::Black), Square::E7);
    }

    #[test]
    fn parse_rejects() {
        for s in ["", "e", "e0", "e9", "i1", "E4", "e44", " e4"] {
            assert_eq!(Square::parse(s), None, "accepted {s:?}");
        }
    }

    const KNIGHT_DELTAS: [(i8, i8); 8] = [
        (1, 2),
        (2, 1),
        (2, -1),
        (1, -2),
        (-1, -2),
        (-2, -1),
        (-2, 1),
        (-1, 2),
    ];

    #[test]
    fn offset_edges() {
        assert_eq!(Square::H1.offset(1, 0), None);
        assert_eq!(Square::A1.offset(-1, 0), None);
        assert_eq!(Square::H8.offset(0, 1), None);
        assert_eq!(Square::E4.offset(1, 2), Some(Square::F6));
        assert_eq!(Square::H8.offset(i8::MAX, i8::MAX), None);
        assert_eq!(Square::A1.offset(i8::MIN, i8::MIN), None);
    }

    #[test]
    fn offset_knight_moves() {
        for sq in Square::ALL {
            for (df, dr) in KNIGHT_DELTAS {
                if let Some(to) = sq.offset(df, dr) {
                    assert_eq!(to.file() as i8 - sq.file() as i8, df, "{sq} -> {to}");
                    assert_eq!(to.rank() as i8 - sq.rank() as i8, dr, "{sq} -> {to}");
                }
            }
        }
    }

    #[test]
    fn knight_move_counts() {
        let count = |sq: Square| {
            KNIGHT_DELTAS
                .into_iter()
                .filter(|&(df, dr)| sq.offset(df, dr).is_some())
                .count()
        };
        assert_eq!(count(Square::A1), 2);
        assert_eq!(count(Square::E4), 8);
    }

    #[test]
    fn forward_backward() {
        assert_eq!(Square::E2.forward(Color::White), Some(Square::E3));
        assert_eq!(Square::E7.forward(Color::Black), Some(Square::E6));
        assert_eq!(Square::E8.forward(Color::White), None);
        assert_eq!(Square::E3.backward(Color::White), Some(Square::E2));
        assert_eq!(Square::E6.backward(Color::Black), Some(Square::E7));
    }

    #[test]
    fn distances() {
        assert_eq!(Square::A1.distance(Square::H8), 7);
        assert_eq!(Square::E4.distance(Square::E4), 0);
        let pairs = [
            (Square::A1, Square::H8),
            (Square::E4, Square::G5),
            (Square::B7, Square::C2),
            (Square::H1, Square::A3),
        ];
        for (a, b) in pairs {
            assert_eq!(a.distance(b), b.distance(a));
        }
    }
}
