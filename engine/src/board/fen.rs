use std::fmt::Write as _;

use super::{Board, CastlingConfig};
use crate::bitboard::Bitboard;
use crate::error::FenError;
use crate::types::{CastleSide, CastlingRights, Color, File, Piece, PieceType, Rank, Square};

impl Board {
    /// The FEN of the standard starting position.
    pub const STARTPOS_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    /// The standard starting position.
    pub fn startpos() -> Board {
        Board::from_fen(Board::STARTPOS_FEN).expect("the starting position is valid")
    }

    /// Reads a FEN string. Castling rights may be standard (`KQkq`), X-FEN
    /// or Shredder-FEN (file letters, for Chess960). The halfmove clock and
    /// fullmove number are optional (0 and 1 by default).
    ///
    /// The en passant square is kept only when a pawn can legally capture
    /// there: otherwise it is dropped, and `to_fen` writes `-`.
    ///
    /// # Errors
    ///
    /// If the string is malformed or describes an impossible position:
    /// see [`FenError`].
    pub fn from_fen(fen: &str) -> Result<Board, FenError> {
        let fields: Vec<&str> = fen.split_whitespace().collect();
        if !(4..=6).contains(&fields.len()) {
            return Err(FenError::FieldCount(fields.len()));
        }
        let mut board = Board::empty();

        parse_placement(&mut board, fields[0])?;

        let mut side = fields[1].chars();
        board.side_to_move = match (side.next(), side.next()) {
            (Some(c), None) => Color::from_char(c),
            _ => None,
        }
        .ok_or(FenError::SideToMove)?;

        (board.castling_rights, board.castling) = parse_castling(&board, fields[2])?;

        if fields[3] != "-" {
            board.en_passant = parse_en_passant(&board, fields[3])?;
        }

        board.halfmove_clock = match fields.get(4) {
            Some(s) => s.parse().map_err(|_| FenError::Counter)?,
            None => 0,
        };
        board.fullmove_number = match fields.get(5) {
            Some(s) => s
                .parse()
                .ok()
                .filter(|&n| n >= 1)
                .ok_or(FenError::Counter)?,
            None => 1,
        };

        board.refresh_hashes();
        board.refresh_checkers();
        let us = board.side_to_move;
        let their_king = board.king_square(!us);
        if !(board.attackers_to(their_king, board.occupied()) & board.color_bb(us)).is_empty() {
            return Err(FenError::OpponentInCheck);
        }
        debug_assert_eq!(board.check_consistency(), Ok(()));
        Ok(board)
    }

    /// Writes the position as FEN. Castling rights use the standard letters
    /// when the rook is the outermost one on its side, its file letter
    /// otherwise (X-FEN), so standard positions give standard FEN.
    #[must_use]
    pub fn to_fen(&self) -> String {
        let mut fen = String::new();
        for rank in Rank::ALL.into_iter().rev() {
            let mut empty = 0;
            for file in File::ALL {
                match self.piece_on(Square::new(file, rank)) {
                    Some(piece) => {
                        if empty > 0 {
                            write!(fen, "{empty}").unwrap();
                            empty = 0;
                        }
                        fen.push(piece.to_char());
                    }
                    None => empty += 1,
                }
            }
            if empty > 0 {
                write!(fen, "{empty}").unwrap();
            }
            if rank != Rank::R1 {
                fen.push('/');
            }
        }

        write!(fen, " {} ", self.side_to_move.to_char()).unwrap();
        self.write_castling(&mut fen);
        match self.en_passant {
            Some(sq) => write!(fen, " {sq}").unwrap(),
            None => fen.push_str(" -"),
        }
        write!(fen, " {} {}", self.halfmove_clock, self.fullmove_number).unwrap();
        fen
    }

    fn write_castling(&self, fen: &mut String) {
        if self.castling_rights.is_empty() {
            fen.push('-');
            return;
        }
        for color in Color::ALL {
            for side in CastleSide::ALL {
                if !self.castling_rights.has(color, side) {
                    continue;
                }
                let rook = self
                    .castling
                    .rook_start(color, side)
                    .expect("a castling right has its rook");
                let rank_rooks =
                    self.piece_bb(color, PieceType::Rook) & Bitboard::from_rank(rook.rank());
                let letter = match side {
                    CastleSide::King if rank_rooks.msb() == rook => 'k',
                    CastleSide::Queen if rank_rooks.lsb() == rook => 'q',
                    _ => rook.file().to_char(),
                };
                fen.push(match color {
                    Color::White => letter.to_ascii_uppercase(),
                    Color::Black => letter,
                });
            }
        }
    }
}

/// Fills `board` from the piece placement field, then checks the kings and
/// the pawns.
fn parse_placement(board: &mut Board, field: &str) -> Result<(), FenError> {
    // `rank` is an index (0 for rank 1), the error holds the rank number.
    let bad_rank = |rank: usize| FenError::Rank(rank as u8 + 1);
    let mut rank = 7;
    let mut file = 0;
    for c in field.chars() {
        match c {
            '/' => {
                if file != 8 || rank == 0 {
                    return Err(bad_rank(rank));
                }
                rank -= 1;
                file = 0;
            }
            '1'..='8' => {
                file += c as usize - '0' as usize;
                if file > 8 {
                    return Err(bad_rank(rank));
                }
            }
            _ => {
                let piece = Piece::from_char(c).ok_or(FenError::UnknownPiece(c))?;
                if file >= 8 {
                    return Err(bad_rank(rank));
                }
                board.add_piece(piece, Square::new(File::ALL[file], Rank::ALL[rank]));
                file += 1;
            }
        }
    }
    if rank != 0 || file != 8 {
        return Err(bad_rank(rank));
    }

    for color in Color::ALL {
        if !board.piece_bb(color, PieceType::King).is_single() {
            return Err(FenError::KingCount);
        }
    }
    let back_ranks = Bitboard::from_rank(Rank::R1) | Bitboard::from_rank(Rank::R8);
    if !(board.pieces(PieceType::Pawn) & back_ranks).is_empty() {
        return Err(FenError::PawnOnBackRank);
    }
    Ok(())
}

/// Reads the castling field: `-`, standard letters (`KQkq`: the outermost
/// rook on that side of the king) or file letters (`HAha`: the rook on that
/// file, its side given by where it stands relative to the king).
fn parse_castling(
    board: &Board,
    field: &str,
) -> Result<(CastlingRights, CastlingConfig), FenError> {
    let kings = Color::ALL.map(|color| board.king_square(color));
    let mut rights = CastlingRights::NONE;
    let mut rooks = [[None; CastleSide::COUNT]; Color::COUNT];
    if field != "-" {
        for c in field.chars() {
            let color = if c.is_ascii_uppercase() {
                Color::White
            } else {
                Color::Black
            };
            let rank = Rank::R1.relative_to(color);
            let king = kings[color.index()];
            if king.rank() != rank {
                return Err(FenError::Castling);
            }
            let mut rank_rooks =
                (board.piece_bb(color, PieceType::Rook) & Bitboard::from_rank(rank)).into_iter();
            let (side, rook) = match c.to_ascii_lowercase() {
                'k' => (
                    CastleSide::King,
                    rank_rooks.filter(|sq| sq.file() > king.file()).max(),
                ),
                'q' => (
                    CastleSide::Queen,
                    rank_rooks.find(|sq| sq.file() < king.file()),
                ),
                letter => {
                    let file = File::from_char(letter).ok_or(FenError::Castling)?;
                    let sq = Square::new(file, rank);
                    let side = if file > king.file() {
                        CastleSide::King
                    } else {
                        CastleSide::Queen
                    };
                    (side, rank_rooks.find(|&rook| rook == sq))
                }
            };
            let rook = rook.ok_or(FenError::Castling)?;
            if rights.has(color, side) {
                return Err(FenError::Castling);
            }
            rights = rights.add(color, side);
            rooks[color.index()][side.index()] = Some(rook);
        }
    }
    Ok((rights, CastlingConfig::new(rooks)))
}

/// Reads the en passant square, once the side to move is known. Returns
/// `None` for a possible square no pawn can legally capture on.
fn parse_en_passant(board: &Board, field: &str) -> Result<Option<Square>, FenError> {
    let us = board.side_to_move;
    let ep = Square::parse(field).ok_or(FenError::EnPassant)?;
    let empty = |sq: Option<Square>| sq.is_some_and(|sq| board.piece_on(sq).is_none());
    // The pawn that just moved two squares stands behind `ep`, and the
    // square it came from, in front of `ep`, is empty.
    let pushed = ep.backward(us).and_then(|sq| board.piece_on(sq));
    if ep.rank() != Rank::R6.relative_to(us)
        || !empty(Some(ep))
        || pushed != Some(Piece::new(!us, PieceType::Pawn))
        || !empty(ep.forward(us))
    {
        return Err(FenError::EnPassant);
    }
    Ok(board.has_legal_en_passant(us, ep).then_some(ep))
}
