use crate::types::{CastleSide, CastlingRights, Color, File, Rank, Square};

/// The game's castling setup (Chess960 included): where each castling rook
/// starts. Fixed when the position is loaded.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CastlingConfig {
    /// Indexed by `Color::index()` then `CastleSide::index()`.
    rook_start: [[Option<Square>; CastleSide::COUNT]; Color::COUNT],
}

impl CastlingConfig {
    /// No castling at all.
    pub(crate) const NONE: CastlingConfig = CastlingConfig {
        rook_start: [[None; CastleSide::COUNT]; Color::COUNT],
    };

    /// Builds the setup from the rooks' starting squares, indexed by
    /// `Color::index()` then `CastleSide::index()`.
    pub(crate) const fn new(
        rooks: [[Option<Square>; CastleSide::COUNT]; Color::COUNT],
    ) -> CastlingConfig {
        CastlingConfig { rook_start: rooks }
    }

    /// The standard setup: rooks in the corners.
    #[cfg(test)]
    pub(crate) const fn standard() -> CastlingConfig {
        CastlingConfig::new([
            [Some(Square::H1), Some(Square::A1)],
            [Some(Square::H8), Some(Square::A8)],
        ])
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
        CastlingConfig { rook_start }
    }

    /// Starting square of `color`'s rook castling on `side`, if any.
    #[inline]
    pub const fn rook_start(&self, color: Color, side: CastleSide) -> Option<Square> {
        self.rook_start[color.index()][side.index()]
    }

    /// The `rights` left after a move from `from` to `to` by a `mover`
    /// piece, `king_moved` if it is the king: a king move drops both of the
    /// mover's rights, and leaving or landing on a castling rook's start
    /// square drops that rook's right.
    ///
    /// The king's start square is not needed: while a side still has a
    /// right, its king has never moved.
    #[inline]
    pub const fn rights_after(
        &self,
        rights: CastlingRights,
        mover: Color,
        king_moved: bool,
        from: Square,
        to: Square,
    ) -> CastlingRights {
        // Most positions of a search have no right left.
        if rights.is_empty() {
            return rights;
        }
        let mut rights = if king_moved {
            rights.remove_color(mover)
        } else {
            rights
        };
        let mut c = 0;
        while c < Color::COUNT {
            let mut s = 0;
            while s < CastleSide::COUNT {
                if let Some(rook) = self.rook_start[c][s]
                    && (rook.index() == from.index() || rook.index() == to.index())
                {
                    rights = rights.remove(Color::ALL[c], CastleSide::ALL[s]);
                }
                s += 1;
            }
            c += 1;
        }
        rights
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

    const ALL: CastlingRights = CastlingRights::ALL;

    #[test]
    fn standard_rights_after() {
        use Color::{Black, White};
        let config = CastlingConfig::standard();
        let after =
            |mover, king_moved, from, to| config.rights_after(ALL, mover, king_moved, from, to);
        // King moves, castles included, drop both rights of the mover.
        assert_eq!(
            after(White, true, Square::E1, Square::E2),
            ALL.remove_color(White)
        );
        assert_eq!(
            after(Black, true, Square::E8, Square::H8),
            ALL.remove_color(Black)
        );
        // A rook leaving its corner drops its own right.
        assert_eq!(
            after(White, false, Square::H1, Square::H5),
            !CastlingRights::WHITE_KING_SIDE
        );
        assert_eq!(
            after(Black, false, Square::A8, Square::A1),
            !(CastlingRights::BLACK_QUEEN_SIDE | CastlingRights::WHITE_QUEEN_SIDE)
        );
        // Capturing a rook in its corner drops the victim's right.
        assert_eq!(
            after(White, false, Square::B7, Square::H8),
            !CastlingRights::BLACK_KING_SIDE
        );
        // Any other move keeps everything.
        assert_eq!(after(White, false, Square::E2, Square::E4), ALL);
        assert_eq!(after(Black, false, Square::G8, Square::F6), ALL);
    }

    #[test]
    fn no_rights_stay_none() {
        let config = CastlingConfig::standard();
        let none = CastlingRights::NONE;
        assert_eq!(
            config.rights_after(none, Color::White, true, Square::E1, Square::H1),
            none
        );
    }

    #[test]
    fn chess960_rights_after() {
        // Rooks on g1 and a1, g8 only.
        let config = CastlingConfig::new([
            [Some(Square::G1), Some(Square::A1)],
            [Some(Square::G8), None],
        ]);
        assert_eq!(config.rook_start(Color::Black, CastleSide::Queen), None);
        assert_eq!(
            config.rook_start(Color::White, CastleSide::King),
            Some(Square::G1)
        );
        let after = |from, to| config.rights_after(ALL, Color::White, false, from, to);
        assert_eq!(
            after(Square::G1, Square::G2),
            !CastlingRights::WHITE_KING_SIDE
        );
        assert_eq!(
            after(Square::C3, Square::G8),
            !CastlingRights::BLACK_KING_SIDE
        );
        assert_eq!(after(Square::H1, Square::H2), ALL);
        assert_eq!(after(Square::C3, Square::A8), ALL);
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
