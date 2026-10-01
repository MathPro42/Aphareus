use super::Board;
use crate::bitboard::Bitboard;
use crate::types::{CastleSide, Color, Piece, PieceType, Rank, Square};

/// Returns early with the formatted message unless `cond` holds.
macro_rules! ensure {
    ($cond:expr, $($msg:tt)+) => {
        if !$cond {
            return Err(format!($($msg)+));
        }
    };
}

impl Board {
    /// Checks every invariant of the board, from the bitboards to the
    /// castling rights, and describes the first one broken. Meant for debug
    /// builds, after every move played, FEN read and mirror.
    pub(crate) fn check_consistency(&self) -> Result<(), String> {
        for (i, a) in self.by_type.iter().enumerate() {
            for (j, b) in self.by_type.iter().enumerate().skip(i + 1) {
                ensure!((*a & *b).is_empty(), "piece types {i} and {j} overlap");
            }
        }
        ensure!(
            (self.by_color[0] & self.by_color[1]).is_empty(),
            "colors overlap"
        );
        let types = self
            .by_type
            .iter()
            .fold(Bitboard::EMPTY, |acc, &bb| acc | bb);
        ensure!(types == self.occupied(), "piece types and colors differ");

        for sq in Square::ALL {
            match self.piece_on(sq) {
                Some(piece) => ensure!(
                    self.piece_bb(piece.color(), piece.piece_type())
                        .contains(sq),
                    "mailbox has {piece} on {sq}, the bitboards do not"
                ),
                None => ensure!(
                    !self.occupied().contains(sq),
                    "bitboards have a piece on {sq}, the mailbox does not"
                ),
            }
        }

        for color in Color::ALL {
            ensure!(
                self.piece_bb(color, PieceType::King).is_single(),
                "{color:?} does not have exactly one king"
            );
        }
        let back_ranks = Bitboard::from_rank(Rank::R1) | Bitboard::from_rank(Rank::R8);
        ensure!(
            (self.pieces(PieceType::Pawn) & back_ranks).is_empty(),
            "pawn on the first or last rank"
        );

        ensure!(
            (self.hash, self.pawn_hash, self.non_pawn_hash) == self.compute_hashes(),
            "hashes differ from a full recomputation"
        );
        ensure!(self.checkers == self.compute_checkers(), "stale checkers");

        let us = self.side_to_move;
        let them = !us;
        ensure!(
            (self.attackers_to(self.king_square(them), self.occupied()) & self.color_bb(us))
                .is_empty(),
            "{them:?} is in check but it is {us:?} to move"
        );

        if let Some(ep) = self.en_passant {
            ensure!(
                ep.rank() == Rank::R6.relative_to(us),
                "en passant square {ep} on the wrong rank"
            );
            ensure!(
                self.piece_on(ep).is_none(),
                "en passant square {ep} is occupied"
            );
            ensure!(
                self.has_legal_en_passant(us, ep),
                "en passant square {ep} set but no legal capture"
            );
        }

        for color in Color::ALL {
            for side in CastleSide::ALL {
                if !self.castling_rights.has(color, side) {
                    continue;
                }
                let rook = self.castling.rook_start(color, side);
                ensure!(
                    rook.and_then(|sq| self.piece_on(sq))
                        == Some(Piece::new(color, PieceType::Rook)),
                    "{color:?} may castle {side:?} side but has no rook there"
                );
                ensure!(
                    self.king_square(color).rank() == Rank::R1.relative_to(color),
                    "{color:?} may castle but its king left its first rank"
                );
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::CastlingConfig;
    use super::*;
    use crate::types::CastlingRights;
    use Piece::*;
    use Square::*;

    fn valid() -> Board {
        let mut board = Board::with_pieces(
            &[
                (WhiteKing, E1),
                (WhiteRook, H1),
                (WhitePawn, E5),
                (BlackKing, E8),
                (BlackRook, A8),
                (BlackPawn, D5),
            ],
            Color::White,
        );
        board.castling = CastlingConfig::standard();
        board.castling_rights = CastlingRights::WHITE_KING_SIDE | CastlingRights::BLACK_QUEEN_SIDE;
        board.en_passant = Some(D6);
        board.refresh_hashes();
        board
    }

    /// Asserts the board is rejected with a message containing `expected`.
    fn assert_invalid(board: &Board, expected: &str) {
        match board.check_consistency() {
            Ok(()) => panic!("accepted {board:?}"),
            Err(msg) => assert!(msg.contains(expected), "{msg:?} lacks {expected:?}"),
        }
    }

    #[test]
    fn valid_board_passes() {
        assert_eq!(valid().check_consistency(), Ok(()));
    }

    #[test]
    fn mailbox_without_bitboard() {
        let mut board = valid();
        board.mailbox[C3.index()] = Some(WhiteKnight);
        assert_invalid(&board, "mailbox has N on c3");
    }

    #[test]
    fn bitboard_without_mailbox() {
        let mut board = valid();
        board.mailbox[E5.index()] = None;
        assert_invalid(&board, "bitboards have a piece on e5");
    }

    #[test]
    fn wrong_hashes() {
        let mut board = valid();
        board.hash ^= 1;
        assert_invalid(&board, "hashes differ");
        let mut board = valid();
        board.pawn_hash ^= 1;
        assert_invalid(&board, "hashes differ");
        let mut board = valid();
        board.non_pawn_hash[1] ^= 1;
        assert_invalid(&board, "hashes differ");
    }

    #[test]
    fn two_white_kings() {
        let mut board = valid();
        board.add_piece(WhiteKing, A1);
        assert_invalid(&board, "White does not have exactly one king");
    }

    #[test]
    fn pawn_on_back_rank() {
        let mut board = valid();
        board.add_piece(BlackPawn, B1);
        assert_invalid(&board, "pawn on the first or last rank");
    }

    #[test]
    fn stale_checkers() {
        let mut board = valid();
        board.checkers = Bitboard::from_square(A8);
        assert_invalid(&board, "stale checkers");
    }

    #[test]
    fn side_not_to_move_in_check() {
        let mut board = valid();
        board.add_piece(WhiteQueen, B5);
        board.refresh_checkers();
        assert_invalid(&board, "Black is in check");
    }

    #[test]
    fn bad_en_passant() {
        for (ep, expected) in [(D3, "wrong rank"), (C6, "no legal capture")] {
            let mut board = valid();
            board.en_passant = Some(ep);
            board.refresh_hashes();
            assert_invalid(&board, expected);
        }
    }

    #[test]
    fn castling_right_without_rook() {
        let mut board = valid();
        board.remove_piece(WhiteRook, H1);
        assert_invalid(&board, "White may castle King side");
    }

    #[test]
    fn castling_right_with_moved_king() {
        // Black keeps its queen side right and its a8 rook.
        let mut board = valid();
        board.move_piece(BlackKing, E8, E7);
        assert_invalid(&board, "Black may castle but its king left");
    }
}
