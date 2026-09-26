use std::fmt;
use std::ops::Not;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    /// Number of color
    pub const COUNT: usize = 2;

    /// List of all the existant color
    pub const ALL: [Color; Color::COUNT] = [Color::White, Color::Black];

    /// Index for array lookups (0 for White, 1 for Black).
    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Returns the other color.
    #[inline]
    pub const fn flip(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    /// Parses a letter (`'w'` or `'b'`) to a Color.
    /// Any other character returns `None`.
    #[inline]
    pub const fn from_char(c: char) -> Option<Color> {
        match c {
            'w' => Some(Color::White),
            'b' => Some(Color::Black),
            _ => None,
        }
    }

    /// Returns the letter (`'w'` or `'b'`) of the assossiated Color.
    #[inline]
    pub const fn to_char(self) -> char {
        match self {
            Color::White => 'w',
            Color::Black => 'b',
        }
    }
}

impl Not for Color {
    type Output = Color;
    fn not(self) -> Color {
        self.flip()
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flip_color() {
        assert_eq!(Color::White.flip(), Color::Black);
        assert_eq!(Color::Black.flip(), Color::White);
    }

    #[test]
    fn double_filp_color() {
        assert_eq!(Color::White.flip().flip(), Color::White);
        assert_eq!(Color::Black.flip().flip(), Color::Black);
    }

    #[test]
    fn all_is_in_index_order() {
        assert_eq!(Color::ALL.len(), Color::COUNT);
        for (i, c) in Color::ALL.into_iter().enumerate() {
            assert_eq!(c.index(), i);
        }
    }

    #[test]
    fn from_fen_char() {
        for c in Color::ALL {
            assert_eq!(Color::from_char(c.to_char()), Some(c));
        }
        assert_eq!(Color::from_char('w'), Some(Color::White));
        assert_eq!(Color::from_char('b'), Some(Color::Black));
    }

    #[test]
    fn invalid_chars_rejected() {
        for ch in ['W', 'x', ' '] {
            assert_eq!(Color::from_char(ch), None, "accepted {ch:?}");
        }
    }

    #[test]
    fn display() {
        assert_eq!(Color::White.to_string(), "w");
        assert_eq!(Color::Black.to_string(), "b");
    }
}
