use engine::{Color, Piece, PieceType, Square};

use super::params::{
    CENTRALIZATION, KING_FILE, KING_RANK, MATERIAL, PAWN_ADVANCE, PAWN_CENTER, ROOK_SEVENTH,
};
use super::taper::Taper;

/// Material plus piece-square value of each piece on each square, signed by
/// color (positive for White), indexed by `Piece::index()` then `Square::index()`.
static PSQT: [[Taper; Square::COUNT]; Piece::COUNT] = build();

/// Material plus piece-square value of `piece` on `sq`, from White's point of view.
#[inline]
pub(crate) fn psqt(piece: Piece, sq: Square) -> Taper {
    PSQT[piece.index()][sq.index()]
}

/// Piece-square value of a piece of type `piece_type` on `relative`, the square
/// seen from its own side (rank 0 = its first rank). Material not included.
pub(crate) const fn piece_square(piece_type: PieceType, relative: Square) -> Taper {
    let file = relative.file().index() as i32;
    let rank = relative.rank().index() as i32;
    let dc = if file < 7 - file { file } else { 7 - file };
    let dr = if rank < 7 - rank { rank } else { 7 - rank };
    let center = dc + dr;

    let t = piece_type.index();
    let mut v = CENTRALIZATION[t].mul(center - 3);
    match piece_type {
        PieceType::Pawn => {
            v = v.add(PAWN_ADVANCE.mul(rank - 1));
            if dc == 3 && (rank == 3 || rank == 4) {
                v = v.add(PAWN_CENTER);
            }
        }
        PieceType::Rook => {
            if rank == 6 {
                v = v.add(ROOK_SEVENTH);
            }
        }
        PieceType::King => {
            v = v.add(KING_RANK.mul(rank)).add(KING_FILE.mul(dc));
        }
        PieceType::Knight | PieceType::Bishop | PieceType::Queen => {}
    }
    v
}

/// The formulas only see relative squares: Black's squares are flipped here,
/// and only here.
const fn build() -> [[Taper; Square::COUNT]; Piece::COUNT] {
    let mut table = [[Taper::ZERO; Square::COUNT]; Piece::COUNT];
    let mut t = 0;
    while t < PieceType::COUNT {
        let piece_type = PieceType::ALL[t];
        let mut i = 0;
        while i < Square::COUNT {
            let sq = Square::ALL[i];
            let white = MATERIAL[t].add(piece_square(piece_type, sq));
            let black = MATERIAL[t].add(piece_square(piece_type, sq.relative_to(Color::Black)));
            table[Piece::new(Color::White, piece_type).index()][i] = white;
            table[Piece::new(Color::Black, piece_type).index()][i] = black.neg();
            i += 1;
        }
        t += 1;
    }
    table
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hce::taper::s;

    fn white(t: PieceType, sq: Square) -> Taper {
        psqt(Piece::new(Color::White, t), sq)
    }

    fn black(t: PieceType, sq: Square) -> Taper {
        psqt(Piece::new(Color::Black, t), sq)
    }

    #[test]
    fn color_antisymmetry() {
        for t in PieceType::ALL {
            for sq in Square::ALL {
                assert_eq!(black(t, sq), -white(t, sq.flip_vertical()), "{t:?} {sq:?}");
            }
        }
    }

    #[test]
    fn orientation() {
        use PieceType::{King, Knight, Pawn};
        assert!(white(Pawn, Square::E6).eg() > white(Pawn, Square::E3).eg());
        assert!(black(Pawn, Square::E3).eg().abs() > black(Pawn, Square::E6).eg().abs());
        assert!(white(Knight, Square::E4).mg() > white(Knight, Square::A1).mg());
        assert!(white(Knight, Square::E4).eg() > white(Knight, Square::A1).eg());
        assert!(white(King, Square::G1).mg() > white(King, Square::E4).mg());
        assert!(white(King, Square::G1).eg() < white(King, Square::E4).eg());
    }

    #[test]
    fn frozen_values() {
        use PieceType::{Bishop, King, Knight, Pawn, Queen, Rook};
        assert_eq!(white(Knight, Square::E4), s(335, 312));
        assert_eq!(white(Pawn, Square::E2), s(100, 120));
        assert_eq!(white(King, Square::G1), s(-5, -12));
        assert_eq!(white(Pawn, Square::E4), s(125, 140));
        assert_eq!(white(Bishop, Square::C1), s(327, 318));
        assert_eq!(white(Rook, Square::A7), s(495, 540));
        assert_eq!(white(Queen, Square::D1), s(950, 1000));
    }

    #[test]
    fn left_right_symmetry() {
        for piece in Piece::ALL {
            for sq in Square::ALL {
                assert_eq!(
                    psqt(piece, sq),
                    psqt(piece, sq.flip_horizontal()),
                    "{piece:?} {sq:?}"
                );
            }
        }
    }
}
