mod leapers;

use crate::bitboard::Bitboard;
use crate::types::{Color, Square};

/// Squares attacked by a knight on `sq`.
#[inline]
pub const fn knight_attacks(sq: Square) -> Bitboard {
    leapers::KNIGHT[sq.index()]
}

/// Squares attacked by a king on `sq`.
#[inline]
pub const fn king_attacks(sq: Square) -> Bitboard {
    leapers::KING[sq.index()]
}

/// Squares attacked by a `color` pawn on `sq` (captures only, not pushes).
#[inline]
pub const fn pawn_attacks(color: Color, sq: Square) -> Bitboard {
    leapers::PAWN[color.index()][sq.index()]
}

/// All squares attacked by the `color` pawns in `pawns`.
#[inline]
pub const fn pawn_attacks_bb(color: Color, pawns: Bitboard) -> Bitboard {
    Bitboard(pawns.up_left(color).0 | pawns.up_right(color).0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{File, Rank};

    fn bb(squares: &[Square]) -> Bitboard {
        squares
            .iter()
            .fold(Bitboard::EMPTY, |b, &sq| b | Bitboard::from_square(sq))
    }

    fn total(attacks: fn(Square) -> Bitboard) -> u32 {
        Square::ALL.into_iter().map(|sq| attacks(sq).count()).sum()
    }

    #[test]
    fn knight_counts() {
        assert_eq!(knight_attacks(Square::A1).count(), 2);
        assert_eq!(knight_attacks(Square::B1).count(), 3);
        assert_eq!(knight_attacks(Square::E4).count(), 8);
        assert_eq!(knight_attacks(Square::H8).count(), 2);
        assert_eq!(total(knight_attacks), 336);
    }

    #[test]
    fn king_counts() {
        assert_eq!(king_attacks(Square::A1).count(), 3);
        assert_eq!(king_attacks(Square::E1).count(), 5);
        assert_eq!(king_attacks(Square::E4).count(), 8);
        assert_eq!(total(king_attacks), 420);
    }

    #[test]
    fn pawn_specific() {
        assert_eq!(pawn_attacks(Color::White, Square::A2), bb(&[Square::B3]));
        assert_eq!(
            pawn_attacks(Color::White, Square::E2),
            bb(&[Square::D3, Square::F3])
        );
        assert_eq!(pawn_attacks(Color::White, Square::H2), bb(&[Square::G3]));
        assert_eq!(
            pawn_attacks(Color::Black, Square::E7),
            bb(&[Square::D6, Square::F6])
        );
    }

    #[test]
    fn reciprocity() {
        for a in Square::ALL {
            for b in Square::ALL {
                assert_eq!(
                    knight_attacks(a).contains(b),
                    knight_attacks(b).contains(a),
                    "knight {a} {b}"
                );
                assert_eq!(
                    king_attacks(a).contains(b),
                    king_attacks(b).contains(a),
                    "king {a} {b}"
                );
                assert_eq!(
                    pawn_attacks(Color::White, a).contains(b),
                    pawn_attacks(Color::Black, b).contains(a),
                    "pawn {a} {b}"
                );
            }
        }
    }

    #[test]
    fn pawn_color_symmetry() {
        for sq in Square::ALL {
            assert_eq!(
                pawn_attacks(Color::Black, sq),
                pawn_attacks(Color::White, sq.flip_vertical()).flip_vertical(),
                "{sq}"
            );
        }
    }

    #[test]
    fn pawn_attacks_bb_is_union() {
        let sets = [
            Bitboard::EMPTY,
            Bitboard::FULL,
            Bitboard::from_rank(Rank::R2),
            Bitboard::from_rank(Rank::R7),
            Bitboard::from_file(File::A),
            Bitboard::from_file(File::H),
            bb(&[Square::A2, Square::D4, Square::E5, Square::H7]),
        ];
        for pawns in sets {
            for color in Color::ALL {
                let expected = pawns
                    .into_iter()
                    .fold(Bitboard::EMPTY, |b, sq| b | pawn_attacks(color, sq));
                assert_eq!(pawn_attacks_bb(color, pawns), expected, "{color}");
            }
        }
    }
}
