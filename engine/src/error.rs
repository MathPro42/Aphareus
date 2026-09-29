use std::error::Error;
use std::fmt;

/// FEN Error handling
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FenError {
    /// Not 4, 5 or 6 space-separated fields (holds how many there were).
    FieldCount(usize),
    /// A rank of the wrong length, or too many or too few ranks (holds the
    /// rank number, 1 to 8).
    Rank(u8),
    /// A character that is neither a piece letter, a digit nor `/`.
    UnknownPiece(char),
    /// The side to move is not `w` or `b`.
    SideToMove,
    /// The castling field is malformed, or names a missing rook or a king
    /// off its first rank.
    Castling,
    /// The en passant square is malformed or impossible.
    EnPassant,
    /// The halfmove clock or fullmove number is not a valid number.
    Counter,
    /// A color does not have exactly one king.
    KingCount,
    /// A pawn stands on the first or last rank.
    PawnOnBackRank,
    /// The side not to move is in check.
    OpponentInCheck,
}

impl fmt::Display for FenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FenError::FieldCount(n) => write!(f, "expected 4 to 6 fields, found {n}"),
            FenError::Rank(n) => write!(
                f,
                "rank {n} has the wrong length, or the rank count is wrong"
            ),
            FenError::UnknownPiece(c) => write!(f, "unknown piece {c:?}"),
            FenError::SideToMove => write!(f, "side to move must be 'w' or 'b'"),
            FenError::Castling => write!(f, "invalid castling rights"),
            FenError::EnPassant => write!(f, "invalid en passant square"),
            FenError::Counter => write!(f, "invalid halfmove clock or fullmove number"),
            FenError::KingCount => write!(f, "each side needs exactly one king"),
            FenError::PawnOnBackRank => write!(f, "pawn on the first or last rank"),
            FenError::OpponentInCheck => write!(f, "the side not to move is in check"),
        }
    }
}

impl Error for FenError {}
