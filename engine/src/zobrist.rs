use crate::prng::Prng;
use crate::types::{CastlingRights, File, Piece, Square};

const SEED: u64 = 0xD1B5_4A32_D192_ED03;

struct ZobristKeys {
    /// Indexed by `Piece::index()` then `Square::index()`.
    piece_square: [[u64; Square::COUNT]; Piece::COUNT],
    /// Indexed by `CastlingRights::index()`: the XOR of one key per right.
    castling: [u64; CastlingRights::COUNT],
    /// Indexed by `File::index()`.
    en_passant: [u64; File::COUNT],
    /// Applied when Black is to move.
    side: u64,
}

static KEYS: ZobristKeys = build();

const fn build() -> ZobristKeys {
    let mut prng = Prng::new(SEED);

    let mut piece_square = [[0; Square::COUNT]; Piece::COUNT];
    let mut p = 0;
    while p < Piece::COUNT {
        let mut sq = 0;
        while sq < Square::COUNT {
            piece_square[p][sq] = prng.next_u64();
            sq += 1;
        }
        p += 1;
    }

    // One key per right (K, Q, k, q, in bit order); each combination is the
    // XOR of its rights' keys, so `castling[NONE] == 0`.
    let base = [
        prng.next_u64(),
        prng.next_u64(),
        prng.next_u64(),
        prng.next_u64(),
    ];
    let mut castling = [0; CastlingRights::COUNT];
    let mut i = 0;
    while i < CastlingRights::COUNT {
        let mut bit = 0;
        while bit < base.len() {
            if i & (1 << bit) != 0 {
                castling[i] ^= base[bit];
            }
            bit += 1;
        }
        i += 1;
    }

    let mut en_passant = [0; File::COUNT];
    let mut f = 0;
    while f < File::COUNT {
        en_passant[f] = prng.next_u64();
        f += 1;
    }

    ZobristKeys {
        piece_square,
        castling,
        en_passant,
        side: prng.next_u64(),
    }
}

/// Key of `piece` standing on `sq`.
#[inline]
pub const fn piece_square(piece: Piece, sq: Square) -> u64 {
    KEYS.piece_square[piece.index()][sq.index()]
}

/// Key of the castling `rights` as a whole: XOR out the old rights' key and
/// XOR in the new one when they change.
#[inline]
pub const fn castling(rights: CastlingRights) -> u64 {
    KEYS.castling[rights.index()]
}

/// Key of an en passant square on `file`.
#[inline]
pub const fn en_passant(file: File) -> u64 {
    KEYS.en_passant[file.index()]
}

/// Key applied when Black is to move.
#[inline]
pub const fn side() -> u64 {
    KEYS.side
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn all_rights() -> impl Iterator<Item = CastlingRights> {
        (0..16).map(|b| CastlingRights::from_bits(b).unwrap())
    }

    #[test]
    fn keys_are_non_zero_and_distinct() {
        let pieces = Piece::ALL
            .into_iter()
            .flat_map(|p| Square::ALL.into_iter().map(move |sq| piece_square(p, sq)));
        let rights = [
            CastlingRights::WHITE_KING_SIDE,
            CastlingRights::WHITE_QUEEN_SIDE,
            CastlingRights::BLACK_KING_SIDE,
            CastlingRights::BLACK_QUEEN_SIDE,
        ]
        .map(castling);
        let files = File::ALL.map(en_passant);
        let keys: Vec<u64> = pieces.chain(rights).chain(files).chain([side()]).collect();

        assert_eq!(keys.len(), 12 * 64 + 4 + 8 + 1);
        assert!(!keys.contains(&0));
        assert_eq!(keys.iter().collect::<HashSet<_>>().len(), keys.len());
    }

    #[test]
    fn castling_keys_combine() {
        assert_eq!(castling(CastlingRights::NONE), 0);
        for a in all_rights() {
            for b in all_rights() {
                if (a & b).is_empty() {
                    assert_eq!(castling(a | b), castling(a) ^ castling(b), "{a} {b}");
                }
            }
        }
    }

    #[test]
    fn keys_are_frozen() {
        assert_eq!(
            piece_square(Piece::WhitePawn, Square::A1),
            0x015F_6958_028F_9348
        );
    }
}
