use super::Board;
use crate::attacks::{
    between, bishop_attacks, king_attacks, knight_attacks, pawn_attacks, rook_attacks,
};
use crate::bitboard::Bitboard;
use crate::types::{Color, PieceType, Square};
use crate::zobrist;

impl Board {
    /// `(hash, pawn_hash, non_pawn_hash)` recomputed from scratch, for the
    /// FEN reader, the mirror and the validation.
    pub(crate) fn compute_hashes(&self) -> (u64, u64, [u64; Color::COUNT]) {
        let mut hash = 0;
        let mut pawn_hash = 0;
        let mut non_pawn_hash = [0; Color::COUNT];
        for sq in Square::ALL {
            if let Some(piece) = self.piece_on(sq) {
                let key = zobrist::piece_square(piece, sq);
                hash ^= key;
                if piece.piece_type() == PieceType::Pawn {
                    pawn_hash ^= key;
                } else {
                    non_pawn_hash[piece.color().index()] ^= key;
                }
            }
        }
        hash ^= zobrist::castling(self.castling_rights);
        if let Some(ep) = self.en_passant {
            hash ^= zobrist::en_passant(ep.file());
        }
        if self.side_to_move == Color::Black {
            hash ^= zobrist::side();
        }
        (hash, pawn_hash, non_pawn_hash)
    }

    /// Pieces of both colors attacking `sq`, sliders seeing through
    /// everything not in `occupied`.
    pub fn attackers_to(&self, sq: Square, occupied: Bitboard) -> Bitboard {
        let queens = self.pieces(PieceType::Queen);
        (knight_attacks(sq) & self.pieces(PieceType::Knight))
            | (king_attacks(sq) & self.pieces(PieceType::King))
            // A black pawn attacks `sq` from where a white pawn on `sq`
            // would attack, and conversely.
            | (pawn_attacks(Color::White, sq) & self.piece_bb(Color::Black, PieceType::Pawn))
            | (pawn_attacks(Color::Black, sq) & self.piece_bb(Color::White, PieceType::Pawn))
            | (bishop_attacks(sq, occupied) & (self.pieces(PieceType::Bishop) | queens))
            | (rook_attacks(sq, occupied) & (self.pieces(PieceType::Rook) | queens))
    }

    /// Returns `true` if a `capturer` pawn can legally take en passant on
    /// `ep`: some pawn attacks it, and the board after the capture leaves
    /// its king safe (horizontal and diagonal pins, and checks the capture
    /// does not answer, all fall out of that one simulation).
    ///
    /// The en passant square is part of the hash, so it must only be set
    /// when this holds: otherwise the same position gets two hashes.
    pub(crate) fn has_legal_en_passant(&self, capturer: Color, ep: Square) -> bool {
        let pawns = pawn_attacks(!capturer, ep) & self.piece_bb(capturer, PieceType::Pawn);
        if pawns.is_empty() {
            return false;
        }
        let king = self.king_square(capturer);
        let captured = Bitboard::from_square(
            ep.backward(capturer)
                .expect("the en passant square is on the sixth rank"),
        );
        let enemies = self.color_bb(!capturer).without(captured);
        pawns.into_iter().any(|from| {
            let occupied = self.occupied()
                ^ Bitboard::from_square(from)
                ^ captured
                ^ Bitboard::from_square(ep);
            (self.attackers_to(king, occupied) & enemies).is_empty()
        })
    }

    /// Enemy pieces giving check to the side to move.
    pub(crate) fn compute_checkers(&self) -> Bitboard {
        let us = self.side_to_move;
        self.attackers_to(self.king_square(us), self.occupied()) & self.color_bb(!us)
    }

    /// Pieces of the side to move alone between their king and an enemy
    /// slider.
    ///
    /// Computed from scratch on each call, not stored: most positions of a
    /// search never generate moves, so `make_move` does not pay for it.
    /// Move generation computes it once per position, in its `Context`.
    pub fn compute_pinned(&self) -> Bitboard {
        let us = self.side_to_move;
        let them = !us;
        let king = self.king_square(us);
        let queens = self.piece_bb(them, PieceType::Queen);
        // Enemy sliders that would see the king if our pieces were
        // transparent.
        let snipers = (rook_attacks(king, self.color_bb(them))
            & (self.piece_bb(them, PieceType::Rook) | queens))
            | (bishop_attacks(king, self.color_bb(them))
                & (self.piece_bb(them, PieceType::Bishop) | queens));
        let mut pinned = Bitboard::EMPTY;
        for sniper in snipers {
            let blockers = between(king, sniper) & self.occupied();
            if blockers.is_single() && !(blockers & self.color_bb(us)).is_empty() {
                pinned |= blockers;
            }
        }
        pinned
    }

    /// Recomputes the hashes from scratch after the side, rights or en
    /// passant square were set directly.
    pub(crate) fn refresh_hashes(&mut self) {
        (self.hash, self.pawn_hash, self.non_pawn_hash) = self.compute_hashes();
    }

    /// Recomputes the pieces giving check to the side to move.
    pub(crate) fn refresh_checkers(&mut self) {
        self.checkers = self.compute_checkers();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Piece::{self, *};
    use Square::*;

    fn bb(squares: &[Square]) -> Bitboard {
        squares
            .iter()
            .fold(Bitboard::EMPTY, |b, &sq| b | Bitboard::from_square(sq))
    }

    fn white_to_move(pieces: &[(Piece, Square)]) -> Board {
        let mut all = vec![(WhiteKing, E1), (BlackKing, H8)];
        all.extend_from_slice(pieces);
        Board::with_pieces(&all, Color::White)
    }

    #[test]
    fn attackers_to_sees_through_removed_pieces() {
        // d4 is attacked by a white pawn (c3), a black pawn (c5), a black
        // knight (f5), a black rook (d6) hiding a white one (d8), and a
        // white bishop (f6) with a queen behind it (g7).
        let board = Board::with_pieces(
            &[
                (WhiteKing, A1),
                (BlackKing, A8),
                (WhitePawn, C3),
                (BlackPawn, C5),
                (BlackKnight, F5),
                (WhiteBishop, F6),
                (WhiteQueen, G7),
                (WhiteRook, D8),
                (BlackRook, D6),
            ],
            Color::White,
        );
        let direct = bb(&[C3, C5, F5, F6, D6]);
        assert_eq!(board.attackers_to(D4, board.occupied()), direct);
        let without_bishop = board.occupied() ^ Bitboard::from_square(F6);
        assert_eq!(
            board.attackers_to(D4, without_bishop),
            direct | Bitboard::from_square(G7)
        );
    }

    #[test]
    fn knight_pinned_by_rook() {
        let board = white_to_move(&[(WhiteKnight, E4), (BlackRook, E8)]);
        assert_eq!(board.compute_pinned(), Bitboard::from_square(E4));
    }

    #[test]
    fn diagonal_pins() {
        let board = white_to_move(&[
            (WhitePawn, D2),
            (BlackBishop, A5),
            (WhiteRook, F2),
            (BlackQueen, H4),
        ]);
        assert_eq!(board.compute_pinned(), bb(&[D2, F2]));
    }

    #[test]
    fn enemy_blocker_is_not_pinned() {
        let board = white_to_move(&[(BlackKnight, E4), (BlackRook, E8)]);
        assert!(board.compute_pinned().is_empty());
    }

    #[test]
    fn piece_between_two_enemy_pieces_is_not_pinned() {
        let board = white_to_move(&[(BlackPawn, E3), (WhiteKnight, E5), (BlackRook, E8)]);
        assert!(board.compute_pinned().is_empty());
    }

    #[test]
    fn two_blockers_are_not_pinned() {
        let board = white_to_move(&[(WhiteKnight, E3), (WhiteBishop, E5), (BlackRook, E8)]);
        assert!(board.compute_pinned().is_empty());
    }

    #[test]
    fn no_check() {
        let board = white_to_move(&[(BlackRook, D8)]);
        assert!(board.checkers().is_empty());
        assert!(!board.in_check());
    }

    #[test]
    fn single_check() {
        let board = white_to_move(&[(BlackRook, E8), (BlackKnight, A1)]);
        assert_eq!(board.checkers(), Bitboard::from_square(E8));
        assert!(board.in_check());
    }

    #[test]
    fn double_check() {
        let board = white_to_move(&[(BlackKnight, D3), (BlackBishop, B4), (BlackRook, A8)]);
        assert_eq!(board.checkers(), bb(&[D3, B4]));
    }

    #[test]
    fn black_to_move() {
        let board = Board::with_pieces(
            &[
                (WhiteKing, E1),
                (BlackKing, E8),
                (WhiteRook, E2),
                (BlackBishop, E7),
                (WhitePawn, D7),
            ],
            Color::Black,
        );
        assert_eq!(board.checkers(), Bitboard::from_square(D7));
        assert_eq!(board.compute_pinned(), Bitboard::from_square(E7));
    }
}
