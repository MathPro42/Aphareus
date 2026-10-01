use super::masks::Context;
use crate::attacks::{attacks, line};
use crate::bitboard::Bitboard;
use crate::board::Board;
use crate::moves::{Move, MoveSink};
use crate::types::PieceType;

/// Moves of the knights, bishops, rooks or queens to `targets`.
pub(super) fn generate(
    board: &Board,
    ctx: &Context,
    piece: PieceType,
    targets: Bitboard,
    sink: &mut impl MoveSink,
) {
    let pinned = ctx.pinned;
    let mut movers = board.piece_bb(ctx.us, piece);
    if piece == PieceType::Knight {
        movers = movers.without(pinned);
    }
    for from in movers {
        let mut targets = attacks(piece, ctx.us, from, ctx.occupied) & targets;
        if pinned.contains(from) {
            targets &= line(ctx.king, from);
        }
        for to in targets {
            sink.push(if board.piece_on(to).is_some() {
                Move::capture(from, to)
            } else {
                Move::quiet(from, to)
            });
        }
    }
}
