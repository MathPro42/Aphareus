use super::GenMode;
use super::masks::Context;
use crate::attacks::{line, pawn_attacks};
use crate::bitboard::Bitboard;
use crate::moves::{Move, MoveSink};
use crate::types::{Color, PieceType, Rank, Square};

/// Index offsets of `Bitboard::up`, `up_left` and `up_right` for `color`.
const fn offsets(color: Color) -> (i8, i8, i8) {
    match color {
        Color::White => (8, 7, 9),
        Color::Black => (-8, -7, -9),
    }
}

/// The square `offset` behind `to`.
fn origin(to: Square, offset: i8) -> Square {
    Square::from_index_bounded((to.index() as i32 - i32::from(offset)) as u32)
}

/// Pawn moves: the unpinned pawns in bulk, by shifting bitboards, then the
/// pinned ones one by one, then en passant.
pub(super) fn generate<M: GenMode>(ctx: &Context, sink: &mut impl MoveSink) {
    let board = ctx.board;
    let us = ctx.us;
    let (push, left, right) = offsets(us);
    let pawns = board.piece_bb(us, PieceType::Pawn);
    let free = pawns.without(ctx.pinned);
    let seventh = Bitboard::from_rank(Rank::R7.relative_to(us));
    let third = Bitboard::from_rank(Rank::R3.relative_to(us));
    let empty = !ctx.occupied;
    let enemies = board.color_bb(ctx.them) & ctx.check_mask;

    let movers = free.without(seventh);
    if M::QUIET {
        let single = movers.up(us) & empty;
        let double = (single & third).up(us) & empty & ctx.check_mask;
        for to in single & ctx.check_mask {
            sink.push(Move::quiet(origin(to, push), to));
        }
        for to in double {
            sink.push(Move::double_push(origin(to, 2 * push), to));
        }
    }
    if M::NOISY {
        for to in movers.up_left(us) & enemies {
            sink.push(Move::capture(origin(to, left), to));
        }
        for to in movers.up_right(us) & enemies {
            sink.push(Move::capture(origin(to, right), to));
        }
    }

    let promoting = free & seventh;
    if !promoting.is_empty() {
        for to in promoting.up(us) & empty & ctx.check_mask {
            promotions::<M>(origin(to, push), to, false, sink);
        }
        for to in promoting.up_left(us) & enemies {
            promotions::<M>(origin(to, left), to, true, sink);
        }
        for to in promoting.up_right(us) & enemies {
            promotions::<M>(origin(to, right), to, true, sink);
        }
    }

    for from in pawns & ctx.pinned {
        generate_pinned::<M>(ctx, from, sink);
    }

    if M::NOISY
        && let Some(ep) = board.en_passant()
    {
        en_passant(ctx, ep, sink);
    }
}

/// Moves of a pinned pawn.
fn generate_pinned<M: GenMode>(ctx: &Context, from: Square, sink: &mut impl MoveSink) {
    let board = ctx.board;
    let us = ctx.us;
    let allowed = line(ctx.king, from) & ctx.check_mask;
    let promotes = from.rank() == Rank::R7.relative_to(us);
    let is_empty = |sq: Square| board.piece_on(sq).is_none();

    if let Some(to) = from.forward(us).filter(|&to| is_empty(to)) {
        if allowed.contains(to) {
            if promotes {
                promotions::<M>(from, to, false, sink);
            } else if M::QUIET {
                sink.push(Move::quiet(from, to));
            }
        }
        if M::QUIET
            && from.rank() == Rank::R2.relative_to(us)
            && let Some(to2) = to.forward(us)
            && is_empty(to2)
            && allowed.contains(to2)
        {
            sink.push(Move::double_push(from, to2));
        }
    }

    for to in pawn_attacks(us, from) & board.color_bb(ctx.them) & allowed {
        if promotes {
            promotions::<M>(from, to, true, sink);
        } else if M::NOISY {
            sink.push(Move::capture(from, to));
        }
    }
}

/// The promotions from `from` to `to` that belong to mode `M`.
fn promotions<M: GenMode>(from: Square, to: Square, capture: bool, sink: &mut impl MoveSink) {
    for piece in PieceType::PROMOTIONS {
        let noisy = capture || piece == PieceType::Queen;
        if (noisy && M::NOISY) || (!noisy && M::QUIET) {
            sink.push(Move::promotion(from, to, piece, capture));
        }
    }
}

/// En passant captures onto `ep`.
fn en_passant(ctx: &Context, ep: Square, sink: &mut impl MoveSink) {
    let board = ctx.board;
    for from in pawn_attacks(ctx.them, ep) & board.piece_bb(ctx.us, PieceType::Pawn) {
        if board.is_en_passant_legal(ctx.us, from, ep) {
            sink.push(Move::en_passant(from, ep));
        }
    }
}
