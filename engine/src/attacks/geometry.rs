use crate::bitboard::Bitboard;
use crate::types::Square;

type SquarePairTable = [[Bitboard; Square::COUNT]; Square::COUNT];

/// The eight directions, clockwise from north: `DIRECTIONS[(d + 4) % 8]` is
/// the opposite of `DIRECTIONS[d]`.
const DIRECTIONS: [(i8, i8); 8] = [
    (0, 1),
    (1, 1),
    (1, 0),
    (1, -1),
    (0, -1),
    (-1, -1),
    (-1, 0),
    (-1, 1),
];

/// Squares from `a` to the edge in each direction, `a` excluded, indexed by
/// `Square::index()` then direction.
const RAYS: [[Bitboard; DIRECTIONS.len()]; Square::COUNT] = {
    let mut rays = [[Bitboard::EMPTY; DIRECTIONS.len()]; Square::COUNT];
    let mut a = 0;
    while a < Square::COUNT {
        let mut d = 0;
        while d < DIRECTIONS.len() {
            let (df, dr) = DIRECTIONS[d];
            let mut sq = Square::ALL[a];
            while let Some(next) = sq.offset(df, dr) {
                rays[a][d] = Bitboard(rays[a][d].0 | Bitboard::from_square(next).0);
                sq = next;
            }
            d += 1;
        }
        a += 1;
    }
    rays
};

/// `(BETWEEN, LINE)`, filled together by walking every ray.
const fn tables() -> (SquarePairTable, SquarePairTable) {
    let mut between = [[Bitboard::EMPTY; Square::COUNT]; Square::COUNT];
    let mut line = [[Bitboard::EMPTY; Square::COUNT]; Square::COUNT];
    let mut a = 0;
    while a < Square::COUNT {
        let mut d = 0;
        while d < DIRECTIONS.len() {
            let (df, dr) = DIRECTIONS[d];
            let opposite = (d + 4) % DIRECTIONS.len();
            let full = Bitboard(
                RAYS[a][d].0 | RAYS[a][opposite].0 | Bitboard::from_square(Square::ALL[a]).0,
            );
            let mut inside = Bitboard::EMPTY;
            let mut sq = Square::ALL[a];
            while let Some(b) = sq.offset(df, dr) {
                between[a][b.index()] = inside;
                line[a][b.index()] = full;
                inside = Bitboard(inside.0 | Bitboard::from_square(b).0);
                sq = b;
            }
            d += 1;
        }
        a += 1;
    }
    (between, line)
}

/// Squares strictly between `a` and `b` when they share a rank, file or
/// diagonal (neither end included), empty otherwise. Indexed by
/// `Square::index()` twice.
pub(super) static BETWEEN: SquarePairTable = tables().0;

/// The whole rank, file or diagonal through `a` and `b`, edge to edge, when
/// they are aligned, empty otherwise (and when `a == b`). Indexed by
/// `Square::index()` twice.
pub(super) static LINE: SquarePairTable = tables().1;
