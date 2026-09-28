use crate::bitboard::Bitboard;
use crate::types::{Color, Square};

const KNIGHT_JUMPS: [(i8, i8); 8] = [
    (1, 2),
    (2, 1),
    (2, -1),
    (1, -2),
    (-1, -2),
    (-2, -1),
    (-2, 1),
    (-1, 2),
];

const KING_JUMPS: [(i8, i8); 8] = [
    (0, 1),
    (1, 1),
    (1, 0),
    (1, -1),
    (0, -1),
    (-1, -1),
    (-1, 0),
    (-1, 1),
];

const WHITE_PAWN_JUMPS: [(i8, i8); 2] = [(-1, 1), (1, 1)];
const BLACK_PAWN_JUMPS: [(i8, i8); 2] = [(-1, -1), (1, -1)];

/// Squares reached by a knight, indexed by `Square::index()`.
pub(super) static KNIGHT: [Bitboard; Square::COUNT] = jump_table(&KNIGHT_JUMPS);

/// Squares reached by a king, indexed by `Square::index()`.
pub(super) static KING: [Bitboard; Square::COUNT] = jump_table(&KING_JUMPS);

/// Squares attacked by a pawn, indexed by `Color::index()` then
/// `Square::index()`: captures only, not pushes.
pub(super) static PAWN: [[Bitboard; Square::COUNT]; Color::COUNT] =
    [jump_table(&WHITE_PAWN_JUMPS), jump_table(&BLACK_PAWN_JUMPS)];

/// For each square, the union of the squares reached by the given
/// `(file_delta, rank_delta)` jumps that stay on the board.
const fn jump_table(jumps: &[(i8, i8)]) -> [Bitboard; Square::COUNT] {
    let mut table = [Bitboard::EMPTY; Square::COUNT];
    let mut i = 0;
    while i < Square::COUNT {
        let sq = Square::ALL[i];
        let mut j = 0;
        while j < jumps.len() {
            let (df, dr) = jumps[j];
            if let Some(to) = sq.offset(df, dr) {
                table[i] = Bitboard(table[i].0 | Bitboard::from_square(to).0);
            }
            j += 1;
        }
        i += 1;
    }
    table
}
