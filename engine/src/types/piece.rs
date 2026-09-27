use std::fmt;

use crate::types::Color;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(u8)]
pub enum PieceType {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

impl PieceType {
    /// Number of piece types.
    pub const COUNT: usize = 6;

    /// All piece type in order.
    pub const ALL: [PieceType; PieceType::COUNT] = [
        PieceType::Pawn,
        PieceType::Knight,
        PieceType::Bishop,
        PieceType::Rook,
        PieceType::Queen,
        PieceType::King,
    ];

    /// All the possible promotions for a pawn.
    pub const PROMOTIONS: [PieceType; 4] = [
        PieceType::Knight,
        PieceType::Bishop,
        PieceType::Rook,
        PieceType::Queen,
    ];

    /// Index for array lookups.
    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Returns `true` for pieces that slide (bishop, rook, queen).
    #[inline]
    pub const fn is_slider(self) -> bool {
        matches!(self, PieceType::Bishop | PieceType::Rook | PieceType::Queen)
    }

    /// Parse a char to a piece type.
    #[inline]
    pub const fn from_char(c: char) -> Option<PieceType> {
        match c {
            'p' => Some(PieceType::Pawn),
            'n' => Some(PieceType::Knight),
            'b' => Some(PieceType::Bishop),
            'r' => Some(PieceType::Rook),
            'q' => Some(PieceType::Queen),
            'k' => Some(PieceType::King),
            _ => None,
        }
    }

    /// Returns the lowercase piece letter.
    #[inline]
    pub const fn to_char(self) -> char {
        match self {
            PieceType::Pawn => 'p',
            PieceType::Knight => 'n',
            PieceType::Bishop => 'b',
            PieceType::Rook => 'r',
            PieceType::Queen => 'q',
            PieceType::King => 'k',
        }
    }
}

impl fmt::Display for PieceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(u8)]
pub enum Piece {
    WhitePawn = 0,
    BlackPawn = 1,
    WhiteKnight = 2,
    BlackKnight = 3,
    WhiteBishop = 4,
    BlackBishop = 5,
    WhiteRook = 6,
    BlackRook = 7,
    WhiteQueen = 8,
    BlackQueen = 9,
    WhiteKing = 10,
    BlackKing = 11,
}

impl Piece {
    /// Number of pieces.
    pub const COUNT: usize = 12;

    /// All piece in order.
    pub const ALL: [Piece; Piece::COUNT] = [
        Piece::WhitePawn,
        Piece::BlackPawn,
        Piece::WhiteKnight,
        Piece::BlackKnight,
        Piece::WhiteBishop,
        Piece::BlackBishop,
        Piece::WhiteRook,
        Piece::BlackRook,
        Piece::WhiteQueen,
        Piece::BlackQueen,
        Piece::WhiteKing,
        Piece::BlackKing,
    ];

    /// Index for array lookups (0..12).
    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    /// Builds a piece from its color and type (`index = type × 2 + color`).
    #[inline]
    pub const fn new(color: Color, piece_type: PieceType) -> Piece {
        Piece::ALL[piece_type.index() * 2 + color.index()]
    }

    /// Return the piece color.
    #[inline]
    pub const fn color(self) -> Color {
        Color::ALL[self.index() & 1]
    }

    /// Return the piece type.
    #[inline]
    pub const fn piece_type(self) -> PieceType {
        PieceType::ALL[self.index() >> 1]
    }

    /// Change the color of a piece.
    #[inline]
    pub const fn flip(self) -> Piece {
        Piece::ALL[self.index() ^ 1]
    }

    /// Parses a FEN piece letter: uppercase is White (`PNBRQK`),
    /// lowercase is Black (`pnbrqk`). Any other character returns `None`.
    #[inline]
    pub const fn from_char(c: char) -> Option<Piece> {
        let color = if c.is_ascii_uppercase() {
            Color::White
        } else {
            Color::Black
        };
        match PieceType::from_char(c.to_ascii_lowercase()) {
            Some(pt) => Some(Piece::new(color, pt)),
            None => None,
        }
    }

    /// Returns the FEN piece letter (uppercase for White, lowercase for Black).
    #[inline]
    pub const fn to_char(self) -> char {
        let c = self.piece_type().to_char();
        match self.color() {
            Color::White => c.to_ascii_uppercase(),
            Color::Black => c,
        }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Piece type

    #[test]
    fn piece_type_in_index() {
        assert_eq!(PieceType::ALL.len(), PieceType::COUNT);
        for (i, pt) in PieceType::ALL.into_iter().enumerate() {
            assert_eq!(pt.index(), i);
        }
    }

    #[test]
    fn piece_type_char() {
        for pt in PieceType::ALL {
            assert_eq!(PieceType::from_char(pt.to_char()), Some(pt));
        }
    }

    #[test]
    fn piece_type_rejects() {
        for ch in ['P', 'K', 'x', '1', ' '] {
            assert_eq!(PieceType::from_char(ch), None, "accepted {ch:?}");
        }
    }

    #[test]
    fn forbidden_promotions() {
        assert!(!PieceType::PROMOTIONS.contains(&PieceType::Pawn));
        assert!(!PieceType::PROMOTIONS.contains(&PieceType::King));
    }

    #[test]
    fn sliders() {
        let sliders: Vec<PieceType> = PieceType::ALL
            .into_iter()
            .filter(|pt| pt.is_slider())
            .collect();
        assert_eq!(
            sliders,
            [PieceType::Bishop, PieceType::Rook, PieceType::Queen]
        );
    }

    #[test]
    fn piece_type_display() {
        let s: String = PieceType::ALL
            .into_iter()
            .map(|pt| pt.to_string())
            .collect();
        assert_eq!(s, "pnbrqk");
    }

    // Piece

    #[test]
    fn construction() {
        for c in Color::ALL {
            for pt in PieceType::ALL {
                let p = Piece::new(c, pt);
                assert_eq!(p.color(), c);
                assert_eq!(p.piece_type(), pt);
            }
        }
    }

    #[test]
    fn letter_encoding() {
        let s: String = Piece::ALL.into_iter().map(Piece::to_char).collect();
        assert_eq!(s, "PpNnBbRrQqKk");
    }

    #[test]
    fn piece_char() {
        for p in Piece::ALL {
            assert_eq!(Piece::from_char(p.to_char()), Some(p));
        }
    }

    #[test]
    fn piece_invalid_chars() {
        for ch in ['x', '1', ' '] {
            assert_eq!(Piece::from_char(ch), None, "accepted {ch:?}");
        }
    }

    #[test]
    fn flip_color() {
        for p in Piece::ALL {
            let f = p.flip();
            assert_eq!(f.piece_type(), p.piece_type());
            assert_eq!(f.color(), p.color().flip());
            assert_eq!(f.flip(), p);
        }
    }

    #[test]
    fn piece_display() {
        let s: String = Piece::ALL.into_iter().map(|p| p.to_string()).collect();
        assert_eq!(s, "PpNnBbRrQqKk");
    }
}
