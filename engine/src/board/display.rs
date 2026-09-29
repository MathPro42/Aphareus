use std::fmt;

use super::Board;
use crate::types::{File, Piece, Rank, Square};

/// The 8 x 8 grid (rank 8 on top, FEN letters, `.` for an empty square),
/// then the FEN, the hash and the checkers.
impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in Rank::ALL.into_iter().rev() {
            write!(f, "{rank} ")?;
            for file in File::ALL {
                let c = self
                    .piece_on(Square::new(file, rank))
                    .map_or('.', Piece::to_char);
                write!(f, " {c}")?;
            }
            writeln!(f)?;
        }
        write!(f, "  ")?;
        for file in File::ALL {
            write!(f, " {file}")?;
        }
        writeln!(f)?;
        writeln!(f)?;
        writeln!(f, "FEN:      {}", self.to_fen())?;
        writeln!(f, "Hash:     {:016X}", self.hash)?;
        write!(f, "Checkers:")?;
        if self.checkers.is_empty() {
            write!(f, " -")?;
        }
        for sq in self.checkers {
            write!(f, " {sq}")?;
        }
        writeln!(f)
    }
}
