use super::GenMode;
use crate::attacks::between;
use crate::bitboard::Bitboard;
use crate::board::Board;
use crate::types::{Color, Square};

pub struct Context<'a> {
    pub(super) board: &'a Board,
    pub(super) us: Color,
    pub(super) them: Color,
    pub(super) king: Square,
    pub(super) occupied: Bitboard,
    pub(super) check_mask: Bitboard,
    pub(super) double_check: bool,
    pub(super) pinned: Bitboard,
}

impl<'a> Context<'a> {
    pub fn new(board: &'a Board) -> Context<'a> {
        let us = board.side_to_move();
        let king = board.king_square(us);
        let checkers = board.checkers();
        let (check_mask, double_check) = match checkers.count() {
            0 => (Bitboard::FULL, false),
            1 => (between(king, checkers.lsb()) | checkers, false),
            _ => (Bitboard::EMPTY, true),
        };
        Context {
            board,
            us,
            them: !us,
            king,
            occupied: board.occupied(),
            check_mask,
            double_check,
            pinned: board.compute_pinned(),
        }
    }
}

/// Destinations allowed by the mode alone: any square but our own pieces,
/// enemy pieces only, or empty squares only.
pub(super) fn mode_targets<M: GenMode>(board: &Board) -> Bitboard {
    let us = board.side_to_move();
    match (M::NOISY, M::QUIET) {
        (true, true) => !board.color_bb(us),
        (true, false) => board.color_bb(!us),
        _ => !board.occupied(),
    }
}
