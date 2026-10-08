mod king;
mod masks;
mod pawns;
mod pieces;

use crate::board::Board;
use crate::moves::{MoveList, MoveSink};
use crate::types::PieceType;
pub use masks::Context;
use masks::mode_targets;

pub trait GenMode: sealed::Sealed {
    /// Captures, en passant, promotions with capture, queen promotions.
    const NOISY: bool;
    /// Everything else.
    const QUIET: bool;
}

/// Every legal move.
pub struct All;
/// Captures, en passant, promotions with capture, queen promotions.
pub struct Noisy;
/// Quiet moves, double pushes, castles, under-promotions without capture.
pub struct Quiet;

impl GenMode for All {
    const NOISY: bool = true;
    const QUIET: bool = true;
}

impl GenMode for Noisy {
    const NOISY: bool = true;
    const QUIET: bool = false;
}

impl GenMode for Quiet {
    const NOISY: bool = false;
    const QUIET: bool = true;
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::All {}
    impl Sealed for super::Noisy {}
    impl Sealed for super::Quiet {}
}

/// Pushes the legal moves of mode `M` in the position of `ctx` into `sink`.
pub fn generate<M: GenMode>(ctx: &Context, sink: &mut impl MoveSink) {
    let allowed = mode_targets::<M>(ctx.board);
    king::generate(ctx, allowed, sink);
    if ctx.double_check {
        // Only the king can move.
        return;
    }
    pawns::generate::<M>(ctx, sink);
    let targets = allowed & ctx.check_mask;
    for piece in [
        PieceType::Knight,
        PieceType::Bishop,
        PieceType::Rook,
        PieceType::Queen,
    ] {
        pieces::generate(ctx, piece, targets, sink);
    }
    if M::QUIET && !ctx.board.in_check() {
        king::generate_castles(ctx, sink);
    }
}

/// All the legal moves, in a new list.
pub fn legal_moves(board: &Board) -> MoveList {
    let mut list = MoveList::new();
    generate::<All>(&Context::new(board), &mut list);
    list
}
