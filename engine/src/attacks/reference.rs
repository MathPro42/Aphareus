/// North, east, south, west, as `(file_delta, rank_delta)`.
const ROOK_DIRECTIONS: [(i32, i32); 4] = [(0, 1), (1, 0), (0, -1), (-1, 0)];

/// North-east, south-east, south-west, north-west, as
/// `(file_delta, rank_delta)`.
const BISHOP_DIRECTIONS: [(i32, i32); 4] = [(1, 1), (1, -1), (-1, -1), (-1, 1)];

/// Squares reached from `sq` by sliding along each direction until the edge
/// or the first occupied square, which is included whatever its color.
fn slide(sq: usize, occupied: u64, directions: &[(i32, i32)]) -> u64 {
    let mut attacks = 0;
    for &(file_delta, rank_delta) in directions {
        let mut file = (sq % 8) as i32;
        let mut rank = (sq / 8) as i32;
        loop {
            file += file_delta;
            rank += rank_delta;
            if !(0..8).contains(&file) || !(0..8).contains(&rank) {
                break;
            }
            let bit = 1u64 << (rank * 8 + file);
            attacks |= bit;
            if occupied & bit != 0 {
                break;
            }
        }
    }
    attacks
}

/// Squares attacked by a rook on `sq` (0..64) given the `occupied` squares.
/// Blockers are included, whatever their color.
pub fn rook_attacks_slow(sq: usize, occupied: u64) -> u64 {
    slide(sq, occupied, &ROOK_DIRECTIONS)
}

/// Squares attacked by a bishop on `sq` (0..64) given the `occupied` squares.
/// Blockers are included, whatever their color.
pub fn bishop_attacks_slow(sq: usize, occupied: u64) -> u64 {
    slide(sq, occupied, &BISHOP_DIRECTIONS)
}
