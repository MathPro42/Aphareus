use super::Board;
use crate::bitboard::Bitboard;
use crate::types::{Piece, PieceType, Square};
use crate::zobrist;

impl Board {
    /// Puts `piece` on the empty square `sq`.
    pub(crate) fn add_piece(&mut self, piece: Piece, sq: Square) {
        assert!(
            self.mailbox[sq.index()].is_none(),
            "add_piece: {sq} is taken"
        );
        let bit = Bitboard::from_square(sq);
        self.by_type[piece.piece_type().index()] |= bit;
        self.by_color[piece.color().index()] |= bit;
        self.mailbox[sq.index()] = Some(piece);
        self.toggle_hashes(piece, sq);
    }

    /// Removes `piece` from `sq`.
    pub(crate) fn remove_piece(&mut self, piece: Piece, sq: Square) {
        assert_eq!(
            self.mailbox[sq.index()],
            Some(piece),
            "remove_piece: {piece} is not on {sq}"
        );
        let bit = Bitboard::from_square(sq);
        self.by_type[piece.piece_type().index()] ^= bit;
        self.by_color[piece.color().index()] ^= bit;
        self.mailbox[sq.index()] = None;
        self.toggle_hashes(piece, sq);
    }

    /// Moves `piece` from `from` to the empty square `to`.
    pub(crate) fn move_piece(&mut self, piece: Piece, from: Square, to: Square) {
        assert_eq!(
            self.mailbox[from.index()],
            Some(piece),
            "move_piece: {piece} is not on {from}"
        );
        assert!(
            self.mailbox[to.index()].is_none(),
            "move_piece: {to} is taken"
        );
        let both = Bitboard::from_square(from) | Bitboard::from_square(to);
        self.by_type[piece.piece_type().index()] ^= both;
        self.by_color[piece.color().index()] ^= both;
        self.mailbox[from.index()] = None;
        self.mailbox[to.index()] = Some(piece);
        // Leaving `from` and landing on `to`, combined into one key.
        self.toggle_key(
            piece,
            zobrist::piece_square(piece, from) ^ zobrist::piece_square(piece, to),
        );
    }

    /// XORs `piece` on `sq` in or out of the hash and of the pawn or
    /// non-pawn hash.
    fn toggle_hashes(&mut self, piece: Piece, sq: Square) {
        self.toggle_key(piece, zobrist::piece_square(piece, sq));
    }

    /// XORs `key`, belonging to `piece`, into the hash and into the pawn or
    /// non-pawn hash.
    fn toggle_key(&mut self, piece: Piece, key: u64) {
        self.hash ^= key;
        if piece.piece_type() == PieceType::Pawn {
            self.pawn_hash ^= key;
        } else {
            self.non_pawn_hash[piece.color().index()] ^= key;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Color;
    use Piece::*;
    use Square::*;

    fn sample() -> Board {
        Board::with_pieces(
            &[
                (WhiteKing, G1),
                (WhiteRook, F1),
                (WhitePawn, E4),
                (BlackKing, G8),
                (BlackQueen, D8),
                (BlackPawn, D5),
            ],
            Color::Black,
        )
    }

    fn assert_hashes(board: &Board) {
        assert_eq!(
            (board.hash, board.pawn_hash, board.non_pawn_hash),
            board.compute_hashes()
        );
    }

    #[test]
    fn add_then_remove_is_identity() {
        let original = sample();
        for piece in Piece::ALL {
            let mut board = original.clone();
            board.add_piece(piece, C3);
            assert_ne!(board, original);
            board.remove_piece(piece, C3);
            assert_eq!(board, original, "{piece}");
        }
    }

    #[test]
    fn move_there_and_back_is_identity() {
        let original = sample();
        for (from, to) in [(E4, E5), (D8, A5), (G1, H1), (D5, D4)] {
            let mut board = original.clone();
            let piece = original.piece_on(from).unwrap();
            board.move_piece(piece, from, to);
            assert_eq!(board.piece_on(from), None);
            assert_eq!(board.piece_on(to), Some(piece));
            board.move_piece(piece, to, from);
            assert_eq!(board, original, "{from} {to}");
        }
    }

    #[test]
    fn primitives_keep_hashes_consistent() {
        let mut board = sample();
        assert_hashes(&board);
        for (i, piece) in Piece::ALL.into_iter().enumerate() {
            let sq = Square::ALL[16 + i];
            board.add_piece(piece, sq);
            assert_hashes(&board);
            board.move_piece(piece, sq, Square::ALL[40 + i]);
            assert_hashes(&board);
        }
        for i in 0..Piece::COUNT {
            board.remove_piece(Piece::ALL[i], Square::ALL[40 + i]);
            assert_hashes(&board);
        }
        assert_eq!(board, sample());
    }
}
