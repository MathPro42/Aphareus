use crate::types::{CastleSide, CastlingRights, Color, File, Rank, Square};

/// The game's castling setup (Chess960 included): where each rook starts and
/// which rights each square guards. Fixed when the position is loaded.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CastlingConfig {
    /// Indexed by `Color::index()` then `CastleSide::index()`.
    rook_start: [[Option<Square>; CastleSide::COUNT]; Color::COUNT],
    /// Rights that remain when a piece leaves or lands on the square.
    masks: [CastlingRights; Square::COUNT],
}

impl CastlingConfig {
    /// No castling at all: every square keeps every right.
    pub(crate) const NONE: CastlingConfig = CastlingConfig {
        rook_start: [[None; CastleSide::COUNT]; Color::COUNT],
        masks: [CastlingRights::ALL; Square::COUNT],
    };

    /// Builds the setup from the `kings` and `rooks` starting squares, the
    /// rooks indexed by `Color::index()` then `CastleSide::index()`.
    pub(crate) const fn new(
        king_start: [Square; Color::COUNT],
        rooks: [[Option<Square>; CastleSide::COUNT]; Color::COUNT],
    ) -> CastlingConfig {
        let mut masks = [CastlingRights::ALL; Square::COUNT];
        let mut c = 0;
        while c < Color::COUNT {
            let color = Color::ALL[c];
            let king = king_start[c].index();
            masks[king] = masks[king].remove_color(color);
            let mut s = 0;
            while s < CastleSide::COUNT {
                if let Some(rook) = rooks[c][s] {
                    masks[rook.index()] = masks[rook.index()].remove(color, CastleSide::ALL[s]);
                }
                s += 1;
            }
            c += 1;
        }
        CastlingConfig {
            rook_start: rooks,
            masks,
        }
    }

    /// The standard setup: kings on e1/e8, rooks in the corners.
    #[cfg(test)]
    pub(crate) const fn standard() -> CastlingConfig {
        CastlingConfig::new(
            [Square::E1, Square::E8],
            [
                [Some(Square::H1), Some(Square::A1)],
                [Some(Square::H8), Some(Square::A8)],
            ],
        )
    }

    /// The same setup with the board flipped vertically and the colors
    /// swapped.
    pub(crate) const fn mirror(&self) -> CastlingConfig {
        let mut rook_start = [[None; CastleSide::COUNT]; Color::COUNT];
        let mut c = 0;
        while c < Color::COUNT {
            let mut s = 0;
            while s < CastleSide::COUNT {
                if let Some(rook) = self.rook_start[c][s] {
                    rook_start[c ^ 1][s] = Some(rook.flip_vertical());
                }
                s += 1;
            }
            c += 1;
        }
        let mut masks = [CastlingRights::NONE; Square::COUNT];
        let mut sq = 0;
        while sq < Square::COUNT {
            masks[Square::ALL[sq].flip_vertical().index()] = self.masks[sq].flip();
            sq += 1;
        }
        CastlingConfig { rook_start, masks }
    }

    /// Starting square of `color`'s rook castling on `side`, if any.
    #[inline]
    pub const fn rook_start(&self, color: Color, side: CastleSide) -> Option<Square> {
        self.rook_start[color.index()][side.index()]
    }

    /// Rights that remain when a piece leaves or lands on `sq`: AND them
    /// with the current rights for both ends of every move.
    #[inline]
    pub const fn mask(&self, sq: Square) -> CastlingRights {
        self.masks[sq.index()]
    }

    /// Where `color`'s king lands when castling on `side`: g1/c1 relative
    /// to `color`, in Chess960 too.
    #[inline]
    pub const fn king_destination(color: Color, side: CastleSide) -> Square {
        let file = match side {
            CastleSide::King => File::G,
            CastleSide::Queen => File::C,
        };
        Square::new(file, Rank::R1.relative_to(color))
    }

    /// Where `color`'s rook lands when castling on `side`: f1/d1 relative
    /// to `color`, in Chess960 too.
    #[inline]
    pub const fn rook_destination(color: Color, side: CastleSide) -> Square {
        let file = match side {
            CastleSide::King => File::F,
            CastleSide::Queen => File::D,
        };
        Square::new(file, Rank::R1.relative_to(color))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const KQ: CastlingRights = CastlingRights::BLACK_KING_SIDE;

    #[test]
    fn standard_masks() {
        let config = CastlingConfig::standard();
        let all = CastlingRights::ALL;
        assert_eq!(config.mask(Square::E1), all.remove_color(Color::White));
        assert_eq!(config.mask(Square::E8), all.remove_color(Color::Black));
        assert_eq!(config.mask(Square::H1), !CastlingRights::WHITE_KING_SIDE);
        assert_eq!(config.mask(Square::A1), !CastlingRights::WHITE_QUEEN_SIDE);
        assert_eq!(config.mask(Square::H8), !KQ);
        assert_eq!(config.mask(Square::A8), !CastlingRights::BLACK_QUEEN_SIDE);
        let guarded = [
            Square::E1,
            Square::E8,
            Square::H1,
            Square::A1,
            Square::H8,
            Square::A8,
        ];
        for sq in Square::ALL {
            if !guarded.contains(&sq) {
                assert_eq!(config.mask(sq), all, "{sq}");
            }
        }
    }

    #[test]
    fn chess960_masks() {
        // King on b1/b8, rooks on a and g.
        let config = CastlingConfig::new(
            [Square::B1, Square::B8],
            [
                [Some(Square::G1), Some(Square::A1)],
                [Some(Square::G8), None],
            ],
        );
        assert_eq!(config.rook_start(Color::Black, CastleSide::Queen), None);
        assert_eq!(
            config.rook_start(Color::White, CastleSide::King),
            Some(Square::G1)
        );
        assert_eq!(config.mask(Square::G1), !CastlingRights::WHITE_KING_SIDE);
        assert_eq!(config.mask(Square::G8), !KQ);
        assert_eq!(config.mask(Square::A8), CastlingRights::ALL);
        assert_eq!(config.mask(Square::H1), CastlingRights::ALL);
    }

    #[test]
    fn destinations() {
        use CastleSide::{King, Queen};
        use Color::{Black, White};
        assert_eq!(CastlingConfig::king_destination(White, King), Square::G1);
        assert_eq!(CastlingConfig::king_destination(White, Queen), Square::C1);
        assert_eq!(CastlingConfig::king_destination(Black, King), Square::G8);
        assert_eq!(CastlingConfig::king_destination(Black, Queen), Square::C8);
        assert_eq!(CastlingConfig::rook_destination(White, King), Square::F1);
        assert_eq!(CastlingConfig::rook_destination(White, Queen), Square::D1);
        assert_eq!(CastlingConfig::rook_destination(Black, King), Square::F8);
        assert_eq!(CastlingConfig::rook_destination(Black, Queen), Square::D8);
    }
}
