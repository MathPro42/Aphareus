use super::GenMode;
use super::masks::{Context, mode_targets};
use crate::attacks::{between, king_attacks};
use crate::bitboard::Bitboard;
use crate::board::{Board, CastlingConfig};
use crate::moves::{Move, MoveSink};
use crate::types::CastleSide;

/// King steps to squares no enemy piece attacks. The check mask does not
/// apply to the king.
pub(super) fn generate<M: GenMode>(board: &Board, ctx: &Context, sink: &mut impl MoveSink) {
    let enemies = board.color_bb(ctx.them);
    // Without the king, it cannot hide behind itself: stepping back along
    // the line of the rook checking it stays attacked.
    let occupied = ctx.occupied ^ Bitboard::from_square(ctx.king);
    for to in king_attacks(ctx.king) & mode_targets::<M>(board) {
        if (board.attackers_to(to, occupied) & enemies).is_empty() {
            sink.push(if enemies.contains(to) {
                Move::capture(ctx.king, to)
            } else {
                Move::quiet(ctx.king, to)
            });
        }
    }
}

/// Castles, when not in check.
pub(super) fn generate_castles(board: &Board, ctx: &Context, sink: &mut impl MoveSink) {
    let enemies = board.color_bb(ctx.them);
    for side in CastleSide::ALL {
        if !board.castling_rights().has(ctx.us, side) {
            continue;
        }
        let king_from = ctx.king;
        let rook_from = board
            .castling_config()
            .rook_start(ctx.us, side)
            .expect("a castling right has its rook");
        let king_to = CastlingConfig::king_destination(ctx.us, side);
        let rook_to = CastlingConfig::rook_destination(ctx.us, side);

        let occupied =
            ctx.occupied ^ Bitboard::from_square(king_from) ^ Bitboard::from_square(rook_from);
        let king_path = between(king_from, king_to) | Bitboard::from_square(king_to);
        let path = king_path | between(rook_from, rook_to) | Bitboard::from_square(rook_to);
        if !(path & occupied).is_empty() {
            continue;
        }
        let attacked = king_path
            .into_iter()
            .any(|sq| !(board.attackers_to(sq, occupied) & enemies).is_empty());
        if !attacked {
            sink.push(Move::castle(king_from, rook_from, side));
        }
    }
}
