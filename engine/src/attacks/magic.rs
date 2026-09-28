use crate::bitboard::Bitboard;

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
