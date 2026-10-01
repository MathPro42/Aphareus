use crate::bitboard::Bitboard;
use crate::types::Square;

/// How to find one square's attacks in a shared table: keep the relevant
/// blockers, multiply by the magic and keep the top bits.
pub(super) struct MagicEntry {
    /// Squares that can block the slider, edges excluded.
    pub(super) mask: u64,
    pub(super) magic: u64,
    /// `64 -` the number of bits in `mask`.
    pub(super) shift: u32,
    /// Where this square's slice starts in the shared table.
    pub(super) offset: u32,
}

impl MagicEntry {
    /// Index in the shared table of the attacks given `occupied`.
    #[inline]
    pub(super) const fn index(&self, occupied: Bitboard) -> usize {
        // Wrapping on purpose: the magic relies on the product overflowing.
        let hash = (occupied.0 & self.mask).wrapping_mul(self.magic) >> self.shift;
        self.offset as usize + hash as usize
    }
}

// `ROOK_ENTRIES`, `BISHOP_ENTRIES` (indexed by `Square::index()`),
// `ROOK_TABLE` and `BISHOP_TABLE`.
include!(concat!(env!("OUT_DIR"), "/magics.rs"));

/// Returns `true` if the squares' slices tile a table of `len` entries:
/// each slice, `2^(64 - shift)` long, starts where the previous one ends,
/// and the last one ends exactly at `len`.
const fn slices_tile(entries: &[MagicEntry; Square::COUNT], len: usize) -> bool {
    let mut end = 0;
    let mut i = 0;
    while i < Square::COUNT {
        let entry = &entries[i];
        if entry.shift == 0 || entry.shift >= 64 || entry.offset as usize != end {
            return false;
        }
        end += 1 << (64 - entry.shift);
        i += 1;
    }
    end == len
}

// The layout the unchecked lookups rely on, checked at compile time.
const _: () = assert!(slices_tile(&ROOK_ENTRIES, ROOK_TABLE.len()));
const _: () = assert!(slices_tile(&BISHOP_ENTRIES, BISHOP_TABLE.len()));
