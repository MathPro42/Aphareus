use super::{Board, CastlingConfig};
use crate::bitboard::Bitboard;
use crate::moves::Move;
use crate::types::{Color, Piece, PieceType};
use crate::zobrist;

impl Board {
    /// The position after `m`, which must be legal here.
    pub fn make_move(&self, m: Move) -> Board {
        let mut next = self.clone();
        let us = self.side_to_move;
        let them = !us;
        let (from, to) = (m.from(), m.to());
        let piece = self
            .piece_on(from)
            .expect("make_move: no piece on the from square");

        //XOR out the old state keys.
        next.hash ^= zobrist::castling(next.castling_rights);
        if let Some(ep) = next.en_passant.take() {
            next.hash ^= zobrist::en_passant(ep.file());
        }

        //Move the pieces.
        if m.is_castle() {
            // Taking both off before putting them back handles every
            // Chess960 castle.
            let side = m.castle_side();
            next.remove_piece(piece, from);
            next.remove_piece(Piece::new(us, PieceType::Rook), to);
            next.add_piece(
                Piece::new(us, PieceType::King),
                CastlingConfig::king_destination(us, side),
            );
            next.add_piece(
                Piece::new(us, PieceType::Rook),
                CastlingConfig::rook_destination(us, side),
            );
        } else {
            if m.is_en_passant() {
                // The captured pawn stands behind the destination.
                let captured = to.backward(us).expect("en passant from the sixth rank");
                next.remove_piece(Piece::new(them, PieceType::Pawn), captured);
            } else if m.is_capture() {
                let captured = self
                    .piece_on(to)
                    .expect("make_move: capture of an empty square");
                next.remove_piece(captured, to);
            }
            if let Some(promoted) = m.promotion_piece() {
                next.remove_piece(piece, from);
                next.add_piece(Piece::new(us, promoted), to);
            } else {
                next.move_piece(piece, from, to);
            }
        }

        //New en passant square, only if a pawn can legally capture there.
        if m.is_double_push() {
            let skipped = from.forward(us).expect("double push from the second rank");
            if next.has_legal_en_passant(them, skipped) {
                next.en_passant = Some(skipped);
            }
        }

        next.castling_rights = self.castling.rights_after(
            self.castling_rights,
            us,
            piece.piece_type() == PieceType::King,
            from,
            to,
        );

        //Counters.
        if piece.piece_type() == PieceType::Pawn || m.is_capture() {
            next.halfmove_clock = 0;
        } else {
            next.halfmove_clock = next.halfmove_clock.saturating_add(1);
        }
        if us == Color::Black {
            next.fullmove_number = next.fullmove_number.saturating_add(1);
        }

        //Side to move and new state keys.
        next.side_to_move = them;
        next.hash ^= zobrist::side() ^ zobrist::castling(next.castling_rights);
        if let Some(ep) = next.en_passant {
            next.hash ^= zobrist::en_passant(ep.file());
        }

        //Checkers of the new side to move.
        next.refresh_checkers();
        debug_assert_eq!(next.check_consistency(), Ok(()), "after {m:?}");
        next
    }

    /// The position with the turn passed, for null-move pruning. The side
    /// to move must not be in check.
    ///
    /// # Panics
    ///
    /// If the side to move is in check, the opponent could
    /// then take the king, and every later `king_square` would be wrong.
    pub fn make_null_move(&self) -> Board {
        assert!(!self.in_check(), "null move while in check");
        let mut next = self.clone();
        if let Some(ep) = next.en_passant.take() {
            next.hash ^= zobrist::en_passant(ep.file());
        }
        next.side_to_move = !next.side_to_move;
        next.hash ^= zobrist::side();
        next.halfmove_clock = next.halfmove_clock.saturating_add(1);
        // The side that passed could not have been giving check.
        next.checkers = Bitboard::EMPTY;
        debug_assert_eq!(next.check_consistency(), Ok(()), "after a null move");
        next
    }
}
