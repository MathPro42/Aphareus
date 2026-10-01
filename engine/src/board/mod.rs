mod castling;
mod display;
mod fen;
mod make;
mod mirror;
mod primitives;
mod state;
mod validate;

pub use castling::CastlingConfig;

use crate::bitboard::Bitboard;
use crate::types::{CastlingRights, Color, Piece, PieceType, Square};

/// A position: the pieces, the game state, its hashes, and the pieces
/// giving check to the side to move.
#[must_use]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Board {
    /// All pawns, all knights… indexed by `PieceType::index()`.
    by_type: [Bitboard; PieceType::COUNT],
    /// All white pieces, all black pieces, indexed by `Color::index()`.
    by_color: [Bitboard; Color::COUNT],
    /// Indexed by `Square::index()`.
    mailbox: [Option<Piece>; Square::COUNT],
    side_to_move: Color,
    castling_rights: CastlingRights,
    /// Only set when an en passant capture is legal.
    en_passant: Option<Square>,
    /// Plies since the last capture or pawn move (fifty-move rule).
    halfmove_clock: u8,
    fullmove_number: u16,
    hash: u64,
    /// XOR of the pawns' keys only.
    pawn_hash: u64,
    /// XOR of each color's other pieces' keys, king included, indexed by
    /// `Color::index()`.
    non_pawn_hash: [u64; Color::COUNT],
    /// Enemy pieces giving check to the side to move.
    checkers: Bitboard,
    castling: CastlingConfig,
}

impl Board {
    /// A board with no piece, White to move, no rights: the starting point
    /// of the FEN reader and of the mirror. Its hashes are consistent (all
    /// zero), but it has no king, so it is not a valid position.
    pub(crate) const fn empty() -> Board {
        Board {
            by_type: [Bitboard::EMPTY; PieceType::COUNT],
            by_color: [Bitboard::EMPTY; Color::COUNT],
            mailbox: [None; Square::COUNT],
            side_to_move: Color::White,
            castling_rights: CastlingRights::NONE,
            en_passant: None,
            halfmove_clock: 0,
            fullmove_number: 1,
            hash: 0,
            pawn_hash: 0,
            non_pawn_hash: [0; Color::COUNT],
            checkers: Bitboard::EMPTY,
            castling: CastlingConfig::NONE,
        }
    }

    /// All pieces of type `piece_type`, both colors.
    #[inline]
    pub const fn pieces(&self, piece_type: PieceType) -> Bitboard {
        self.by_type[piece_type.index()]
    }

    /// All `color` pieces.
    #[inline]
    pub const fn color_bb(&self, color: Color) -> Bitboard {
        self.by_color[color.index()]
    }

    /// The `color` pieces of type `piece_type`.
    #[inline]
    pub const fn piece_bb(&self, color: Color, piece_type: PieceType) -> Bitboard {
        Bitboard(self.by_type[piece_type.index()].0 & self.by_color[color.index()].0)
    }

    /// All pieces.
    #[inline]
    pub const fn occupied(&self) -> Bitboard {
        Bitboard(self.by_color[0].0 | self.by_color[1].0)
    }

    /// The piece on `sq`, if any.
    #[inline]
    #[must_use]
    pub const fn piece_on(&self, sq: Square) -> Option<Piece> {
        self.mailbox[sq.index()]
    }

    /// Square of `color`'s king.
    #[inline]
    #[must_use]
    pub const fn king_square(&self, color: Color) -> Square {
        self.piece_bb(color, PieceType::King).lsb()
    }

    /// Returns `true` if the side to move is in check.
    #[inline]
    #[must_use]
    pub const fn in_check(&self) -> bool {
        !self.checkers.is_empty()
    }

    #[inline]
    #[must_use]
    pub const fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    #[inline]
    #[must_use]
    pub const fn castling_rights(&self) -> CastlingRights {
        self.castling_rights
    }

    /// The en passant square, only when a legal capture there exists.
    #[inline]
    #[must_use]
    pub const fn en_passant(&self) -> Option<Square> {
        self.en_passant
    }

    /// Plies since the last capture or pawn move (fifty-move rule).
    #[inline]
    #[must_use]
    pub const fn halfmove_clock(&self) -> u8 {
        self.halfmove_clock
    }

    #[inline]
    #[must_use]
    pub const fn fullmove_number(&self) -> u16 {
        self.fullmove_number
    }

    /// The Zobrist hash of the whole position.
    #[inline]
    #[must_use]
    pub const fn hash(&self) -> u64 {
        self.hash
    }

    /// The Zobrist hash of the pawns only.
    #[inline]
    #[must_use]
    pub const fn pawn_hash(&self) -> u64 {
        self.pawn_hash
    }

    /// The Zobrist hash of `color`'s pieces other than pawns, king included.
    #[inline]
    #[must_use]
    pub const fn non_pawn_hash(&self, color: Color) -> u64 {
        self.non_pawn_hash[color.index()]
    }

    /// Enemy pieces giving check to the side to move.
    #[inline]
    pub const fn checkers(&self) -> Bitboard {
        self.checkers
    }

    #[inline]
    #[must_use]
    pub const fn castling_config(&self) -> &CastlingConfig {
        &self.castling
    }

    /// Returns `true` if `color` has a piece other than pawns and king.
    #[inline]
    #[must_use]
    pub const fn has_non_pawn_material(&self, color: Color) -> bool {
        let pawns_and_king = self.pieces(PieceType::Pawn).0 | self.pieces(PieceType::King).0;
        self.color_bb(color).0 & !pawns_and_king != 0
    }

    /// Returns `true` if neither side can ever mate: kings alone, a single
    /// minor piece, or only bishops all on the same square color.
    #[must_use]
    pub fn is_insufficient_material(&self) -> bool {
        let heavy_or_pawns = self.pieces(PieceType::Pawn)
            | self.pieces(PieceType::Rook)
            | self.pieces(PieceType::Queen);
        if !heavy_or_pawns.is_empty() {
            return false;
        }
        let bishops = self.pieces(PieceType::Bishop);
        let minors = self.pieces(PieceType::Knight) | bishops;
        if minors.count() <= 1 {
            return true;
        }
        minors == bishops
            && ((bishops & Bitboard::LIGHT_SQUARES).is_empty()
                || (bishops & Bitboard::DARK_SQUARES).is_empty())
    }
}

#[cfg(test)]
impl Board {
    /// A board holding `pieces`, with `side` to move, no rights and no en
    /// passant, its hashes and state computed.
    pub(crate) fn with_pieces(pieces: &[(Piece, Square)], side: Color) -> Board {
        let mut board = Board::empty();
        for &(piece, sq) in pieces {
            board.add_piece(piece, sq);
        }
        board.side_to_move = side;
        board.refresh_hashes();
        board.refresh_checkers();
        board
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Piece::*;
    use Square::*;

    fn material(pieces: &[(Piece, Square)]) -> bool {
        let mut all = vec![(WhiteKing, E1), (BlackKing, E8)];
        all.extend_from_slice(pieces);
        Board::with_pieces(&all, Color::White).is_insufficient_material()
    }

    #[test]
    fn accessors() {
        let board = Board::with_pieces(
            &[
                (WhiteKing, E1),
                (BlackKing, E8),
                (WhitePawn, E2),
                (BlackRook, A8),
            ],
            Color::White,
        );
        assert_eq!(board.piece_on(E2), Some(WhitePawn));
        assert_eq!(board.piece_on(E4), None);
        assert_eq!(board.king_square(Color::Black), E8);
        assert_eq!(board.occupied().count(), 4);
        assert_eq!(board.color_bb(Color::Black).count(), 2);
        assert_eq!(
            board.piece_bb(Color::Black, PieceType::Rook),
            Bitboard::from_square(A8)
        );
        assert_eq!(board.pieces(PieceType::King).count(), 2);
        assert!(!board.has_non_pawn_material(Color::White));
        assert!(board.has_non_pawn_material(Color::Black));
        assert!(!board.in_check());
    }

    #[test]
    fn insufficient_material() {
        assert!(material(&[]));
        assert!(material(&[(WhiteBishop, C1)]));
        assert!(material(&[(BlackKnight, B8)]));
        // c1 and f8 are both dark squares.
        assert!(material(&[(WhiteBishop, C1), (BlackBishop, F8)]));
        assert!(material(&[(WhiteBishop, C1), (WhiteBishop, A3)]));
    }

    #[test]
    fn sufficient_material() {
        assert!(!material(&[(WhiteKnight, B1), (WhiteKnight, G1)]));
        assert!(!material(&[(WhitePawn, A2)]));
        assert!(!material(&[(BlackRook, A8)]));
        assert!(!material(&[(WhiteQueen, D1)]));
        // c1 is dark, c8 is light.
        assert!(!material(&[(WhiteBishop, C1), (BlackBishop, C8)]));
        assert!(!material(&[(WhiteBishop, C1), (BlackKnight, B8)]));
    }
}
