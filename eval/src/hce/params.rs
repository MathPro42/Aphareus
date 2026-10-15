//! Every numeric value of the hand-crafted evaluation.

use engine::PieceType;

use super::taper::{Taper, s};

/// Phase weight of each piece type, indexed by `PieceType::index()`.
pub(crate) const PHASE_WEIGHTS: [i32; PieceType::COUNT] = [0, 1, 1, 2, 4, 0];

/// Phase of the starting material: pure middlegame.
pub(crate) const PHASE_MAX: i32 = 24;

/// Material value of each piece type, indexed by `PieceType::index()`.
pub(crate) const MATERIAL: [Taper; PieceType::COUNT] = [
    s(100, 120),
    s(320, 300),
    s(330, 320),
    s(480, 540),
    s(950, 1000),
    s(0, 0),
];

/// Bonus per step toward the center, indexed by `PieceType::index()`.
/// Multiplied by `center - 3`: from -3 in a corner to +3 on d4, e4, d5, e5.
pub(crate) const CENTRALIZATION: [Taper; PieceType::COUNT] =
    [Taper::ZERO, s(5, 4), s(3, 2), Taper::ZERO, s(1, 3), s(0, 6)];

/// Pawn bonus per rank beyond its second rank.
pub(crate) const PAWN_ADVANCE: Taper = s(5, 10);

/// Bonus for a d or e pawn on its fourth or fifth rank.
pub(crate) const PAWN_CENTER: Taper = s(15, 0);

/// Bonus for a rook on its seventh rank.
pub(crate) const ROOK_SEVENTH: Taper = s(15, 0);

/// King bonus per rank away from its first rank.
pub(crate) const KING_RANK: Taper = s(-15, 0);

/// King bonus per file away from the nearest edge (a or h file).
pub(crate) const KING_FILE: Taper = s(-5, 0);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_max_is_starting_material() {
        // Pieces of each type at the start, both sides.
        let start = [16, 4, 4, 4, 2, 2];
        let sum: i32 = PieceType::ALL
            .iter()
            .map(|t| PHASE_WEIGHTS[t.index()] * start[t.index()])
            .sum();
        assert_eq!(sum, PHASE_MAX);
    }
}
