use std::fmt;

use crate::types::{CastleSide, PieceType, Square};

/// The low 6 bits hold the from square, the next 6 the to square, and the
/// top 4 a flag saying what kind of move it is.
/// In the flag, bit 2 means capture and bit 3 means promotion.
/// ```text
/// 0 quiet         4      capture
/// 1 double push   5      en passant
/// 2 short castle  6, 7   unused
/// 3 long castle   8-11   promotion to N, B, R, Q
///                 12-15  same with a capture
/// ```
/// A castle is stored as the king capturing its own rook.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Move(u16);

impl Move {
    /// "No move": a1 -> a1, never legal.
    pub const NULL: Move = Move(0);

    /// Quiet move.
    pub const QUIET: u8 = 0;
    /// Pawn double push.
    pub const DOUBLE_PUSH: u8 = 1;
    /// King side castle.
    pub const KING_CASTLE: u8 = 2;
    /// Queen side castle.
    pub const QUEEN_CASTLE: u8 = 3;
    /// Capture.
    pub const CAPTURE: u8 = 4;
    /// En passant capture.
    pub const EN_PASSANT: u8 = 5;
    /// Promotion; add the index into [`PieceType::PROMOTIONS`] (0 to 3).
    pub const PROMOTION: u8 = 8;
    /// Promotion with capture; add the index into [`PieceType::PROMOTIONS`] (0 to 3).
    pub const PROMOTION_CAPTURE: u8 = 12;

    const CAPTURE_BIT: u8 = 4;
    const PROMOTION_BIT: u8 = 8;
    /// Promotion to queen without capture.
    const QUEEN_PROMOTION: u8 = Move::PROMOTION + 3;

    /// Packs a move. The flag must be one of the 14 valid codes.
    #[inline]
    pub const fn new(from: Square, to: Square, flag: u8) -> Move {
        debug_assert!(flag < 16 && flag != 6 && flag != 7, "invalid move flag");
        Move(from.index() as u16 | (to.index() as u16) << 6 | (flag as u16) << 12)
    }

    /// A quiet move.
    #[inline]
    pub const fn quiet(from: Square, to: Square) -> Move {
        Move::new(from, to, Move::QUIET)
    }

    /// A pawn double push.
    #[inline]
    pub const fn double_push(from: Square, to: Square) -> Move {
        Move::new(from, to, Move::DOUBLE_PUSH)
    }

    /// A capture (not en passant).
    #[inline]
    pub const fn capture(from: Square, to: Square) -> Move {
        Move::new(from, to, Move::CAPTURE)
    }

    /// An en passant capture.
    #[inline]
    pub const fn en_passant(from: Square, to: Square) -> Move {
        Move::new(from, to, Move::EN_PASSANT)
    }

    /// A castle.
    #[inline]
    pub const fn castle(king: Square, rook: Square, side: CastleSide) -> Move {
        let flag = match side {
            CastleSide::King => Move::KING_CASTLE,
            CastleSide::Queen => Move::QUEEN_CASTLE,
        };
        Move::new(king, rook, flag)
    }

    /// A promotion to `piece`, which must be a knight, bishop, rook or queen.
    #[inline]
    pub const fn promotion(from: Square, to: Square, piece: PieceType, capture: bool) -> Move {
        debug_assert!(
            matches!(
                piece,
                PieceType::Knight | PieceType::Bishop | PieceType::Rook | PieceType::Queen
            ),
            "invalid promotion piece"
        );
        // PROMOTIONS starts at Knight, whose index is 1.
        let piece_bits = (piece.index() - 1) as u8;
        let base = if capture {
            Move::PROMOTION_CAPTURE
        } else {
            Move::PROMOTION
        };
        Move::new(from, to, base | piece_bits)
    }

    /// The from square.
    #[inline]
    pub const fn from(self) -> Square {
        Square::from_index_bounded((self.0 & 63) as u32)
    }

    /// The to square (the rook's square for a castle).
    #[inline]
    pub const fn to(self) -> Square {
        Square::from_index_bounded(((self.0 >> 6) & 63) as u32)
    }

    /// The 4-bit flag.
    #[inline]
    #[must_use]
    pub const fn flag(self) -> u8 {
        (self.0 >> 12) as u8
    }

    /// Returns `true` for [`Move::NULL`].
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    /// Returns `true` for captures, including en passant and capture-promotions.
    #[inline]
    #[must_use]
    pub const fn is_capture(self) -> bool {
        self.flag() & Move::CAPTURE_BIT != 0
    }

    /// Returns `true` for promotions, with or without capture.
    #[inline]
    #[must_use]
    pub const fn is_promotion(self) -> bool {
        self.flag() & Move::PROMOTION_BIT != 0
    }

    /// Returns `true` for both castles.
    #[inline]
    #[must_use]
    pub const fn is_castle(self) -> bool {
        matches!(self.flag(), Move::KING_CASTLE | Move::QUEEN_CASTLE)
    }

    /// Returns `true` for en passant captures.
    #[inline]
    #[must_use]
    pub const fn is_en_passant(self) -> bool {
        self.flag() == Move::EN_PASSANT
    }

    /// Returns `true` for pawn double pushes.
    #[inline]
    #[must_use]
    pub const fn is_double_push(self) -> bool {
        self.flag() == Move::DOUBLE_PUSH
    }

    /// The side of a castle.
    #[inline]
    pub const fn castle_side(self) -> CastleSide {
        debug_assert!(self.is_castle(), "castle_side of a non-castle move");
        if self.flag() == Move::KING_CASTLE {
            CastleSide::King
        } else {
            CastleSide::Queen
        }
    }

    /// The promotion piece, or `None` if the move is not a promotion.
    #[inline]
    #[must_use]
    pub const fn promotion_piece(self) -> Option<PieceType> {
        if !self.is_promotion() {
            return None;
        }
        // Knight to queen are the piece types 1 to 4.
        Some(PieceType::from_index_bounded((self.flag() & 3) as u32 + 1))
    }

    /// Moves searched by the noisy generation mode.
    #[inline]
    #[must_use]
    pub const fn is_noisy(self) -> bool {
        self.is_capture() || self.flag() == Move::QUEEN_PROMOTION
    }

    /// The packed 16 bits.
    #[inline]
    #[must_use]
    pub const fn raw(self) -> u16 {
        self.0
    }

    /// Unpacks 16 bits, for the transposition table. Legality is checked elsewhere.
    #[inline]
    pub const fn from_raw(raw: u16) -> Move {
        Move(raw)
    }
}

/// Shows the squares and the kind of move, such as `e2e4 [double push]`.
impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_null() {
            return write!(f, "NULL");
        }
        let kind = match self.flag() {
            Move::QUIET => "quiet",
            Move::DOUBLE_PUSH => "double push",
            Move::KING_CASTLE => "king castle",
            Move::QUEEN_CASTLE => "queen castle",
            Move::CAPTURE => "capture",
            Move::EN_PASSANT => "en passant",
            6 | 7 => "reserved",
            8..=11 => "promotion",
            _ => "capture-promotion",
        };
        write!(f, "{}{}", self.from(), self.to())?;
        if let Some(piece) = self.promotion_piece() {
            write!(f, "{piece}")?;
        }
        write!(f, " [{kind}]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_FLAGS: [u8; 14] = [0, 1, 2, 3, 4, 5, 8, 9, 10, 11, 12, 13, 14, 15];

    #[test]
    fn round_trip() {
        for from in Square::ALL {
            for to in Square::ALL {
                for flag in VALID_FLAGS {
                    let m = Move::new(from, to, flag);
                    assert_eq!(m.from(), from);
                    assert_eq!(m.to(), to);
                    assert_eq!(m.flag(), flag);
                    assert_eq!(Move::from_raw(m.raw()), m);
                }
            }
        }
    }

    #[test]
    fn predicates() {
        // flag, capture, promotion, castle, en passant, double push, noisy
        #[rustfmt::skip]
        let table: [(u8, bool, bool, bool, bool, bool, bool); 14] = [
            (0,  false, false, false, false, false, false),
            (1,  false, false, false, false, true,  false),
            (2,  false, false, true,  false, false, false),
            (3,  false, false, true,  false, false, false),
            (4,  true,  false, false, false, false, true),
            (5,  true,  false, false, true,  false, true),
            (8,  false, true,  false, false, false, false),
            (9,  false, true,  false, false, false, false),
            (10, false, true,  false, false, false, false),
            (11, false, true,  false, false, false, true),
            (12, true,  true,  false, false, false, true),
            (13, true,  true,  false, false, false, true),
            (14, true,  true,  false, false, false, true),
            (15, true,  true,  false, false, false, true),
        ];
        for (flag, capture, promotion, castle, ep, double, noisy) in table {
            let m = Move::new(Square::E2, Square::E4, flag);
            assert_eq!(m.is_capture(), capture, "flag {flag}");
            assert_eq!(m.is_promotion(), promotion, "flag {flag}");
            assert_eq!(m.is_castle(), castle, "flag {flag}");
            assert_eq!(m.is_en_passant(), ep, "flag {flag}");
            assert_eq!(m.is_double_push(), double, "flag {flag}");
            assert_eq!(m.is_noisy(), noisy, "flag {flag}");
        }
    }

    #[test]
    fn castle_side() {
        let short = Move::castle(Square::E1, Square::H1, CastleSide::King);
        let long = Move::castle(Square::E8, Square::A8, CastleSide::Queen);
        assert_eq!(short.castle_side(), CastleSide::King);
        assert_eq!(long.castle_side(), CastleSide::Queen);
        assert_eq!(short.to(), Square::H1);
    }

    #[test]
    fn promotion_piece() {
        let expected = [
            PieceType::Knight,
            PieceType::Bishop,
            PieceType::Rook,
            PieceType::Queen,
        ];
        for flag in VALID_FLAGS {
            let m = Move::new(Square::A7, Square::A8, flag);
            let piece = if flag >= 8 {
                Some(expected[(flag & 3) as usize])
            } else {
                None
            };
            assert_eq!(m.promotion_piece(), piece, "flag {flag}");
        }
    }

    #[test]
    fn constructors() {
        use Square::*;
        assert_eq!(Move::quiet(G1, F3).flag(), Move::QUIET);
        assert_eq!(Move::double_push(E2, E4).flag(), Move::DOUBLE_PUSH);
        assert_eq!(Move::capture(E4, D5).flag(), Move::CAPTURE);
        assert_eq!(Move::en_passant(E5, D6).flag(), Move::EN_PASSANT);
        for piece in PieceType::PROMOTIONS {
            for capture in [false, true] {
                let m = Move::promotion(B7, A8, piece, capture);
                assert_eq!(m.promotion_piece(), Some(piece));
                assert_eq!(m.is_capture(), capture);
            }
        }
    }

    #[test]
    fn null() {
        assert!(Move::NULL.is_null());
        assert_eq!(Move::NULL.raw(), 0);
        assert_eq!(Move::default(), Move::NULL);
        assert!(!Move::quiet(Square::A1, Square::A2).is_null());
    }

    #[test]
    fn debug() {
        assert_eq!(
            format!("{:?}", Move::double_push(Square::E2, Square::E4)),
            "e2e4 [double push]"
        );
        assert_eq!(
            format!(
                "{:?}",
                Move::promotion(Square::B7, Square::A8, PieceType::Queen, true)
            ),
            "b7a8q [capture-promotion]"
        );
        assert_eq!(format!("{:?}", Move::NULL), "NULL");
    }

    #[test]
    fn size() {
        assert_eq!(size_of::<Move>(), 2);
    }
}
