use super::Board;
use crate::types::Square;

impl Board {
    /// The same position with the board flipped vertically and the colors
    /// swapped: White's pieces become Black's, the side to move changes.
    /// Any symmetric evaluation or search gives the same result on both.
    pub fn mirror(&self) -> Board {
        let mut mirrored = Board::empty();
        for sq in Square::ALL {
            if let Some(piece) = self.piece_on(sq) {
                mirrored.add_piece(piece.flip(), sq.flip_vertical());
            }
        }
        mirrored.side_to_move = !self.side_to_move;
        mirrored.castling_rights = self.castling_rights.flip();
        mirrored.castling = self.castling.mirror();
        mirrored.en_passant = self.en_passant.map(Square::flip_vertical);
        mirrored.halfmove_clock = self.halfmove_clock;
        mirrored.fullmove_number = self.fullmove_number;
        mirrored.refresh_hashes();
        mirrored.refresh_checkers();
        debug_assert_eq!(mirrored.check_consistency(), Ok(()));
        mirrored
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = include_str!("../../tests/data/fen_valid.txt");

    #[test]
    fn mirror_is_consistent() {
        let fens = VALID
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#'));
        for fen in fens {
            let board = Board::from_fen(fen).unwrap();
            let mirrored = board.mirror();
            assert_eq!(mirrored.check_consistency(), Ok(()), "{fen}");
            assert_eq!(mirrored.mirror(), board, "{fen}");
        }
    }
}
