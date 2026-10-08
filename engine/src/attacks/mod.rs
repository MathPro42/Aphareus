mod geometry;
mod leapers;
mod magic;
#[cfg(test)]
mod reference;

use crate::bitboard::Bitboard;
use crate::types::{Color, PieceType, Square};

/// Squares attacked by a knight on `sq`.
#[inline]
pub const fn knight_attacks(sq: Square) -> Bitboard {
    leapers::KNIGHT[sq.index()]
}

/// Squares attacked by a king on `sq`.
#[inline]
pub const fn king_attacks(sq: Square) -> Bitboard {
    leapers::KING[sq.index()]
}

/// Squares attacked by a `color` pawn on `sq` (captures only, not pushes).
#[inline]
pub const fn pawn_attacks(color: Color, sq: Square) -> Bitboard {
    leapers::PAWN[color.index()][sq.index()]
}

/// All squares attacked by the `color` pawns in `pawns`.
#[inline]
pub const fn pawn_attacks_bb(color: Color, pawns: Bitboard) -> Bitboard {
    Bitboard(pawns.up_left(color).0 | pawns.up_right(color).0)
}

/// Squares strictly between `a` and `b` if they share a rank, file or
/// diagonal, empty otherwise. Neither `a` nor `b` is included.
#[inline]
pub const fn between(a: Square, b: Square) -> Bitboard {
    geometry::BETWEEN[a.index()][b.index()]
}

/// The whole rank, file or diagonal through `a` and `b`, edge to edge, if
/// they are aligned, empty otherwise (including when `a == b`).
#[inline]
pub const fn line(a: Square, b: Square) -> Bitboard {
    geometry::LINE[a.index()][b.index()]
}

/// Returns `true` if `c` lies on the line through `a` and `b`.
#[inline]
#[must_use]
pub const fn aligned(a: Square, b: Square, c: Square) -> bool {
    line(a, b).contains(c)
}

/// Squares attacked by a bishop on `sq` given the `occupied` squares.
/// Blockers are included, whatever their color.
#[inline]
pub const fn bishop_attacks(sq: Square, occupied: Bitboard) -> Bitboard {
    let index = magic::BISHOP_ENTRIES[sq.index()].index(occupied);
    debug_assert!(index < magic::BISHOP_TABLE.len());
    // SAFETY: `index` is always in bounds for `BISHOP_TABLE`.
    Bitboard(unsafe { *magic::BISHOP_TABLE.as_ptr().add(index) })
}

/// Squares attacked by a rook on `sq` given the `occupied` squares.
/// Blockers are included, whatever their color.
#[inline]
pub const fn rook_attacks(sq: Square, occupied: Bitboard) -> Bitboard {
    let index = magic::ROOK_ENTRIES[sq.index()].index(occupied);
    debug_assert!(index < magic::ROOK_TABLE.len());
    // SAFETY: `index` is always in bounds for `ROOK_TABLE`.
    Bitboard(unsafe { *magic::ROOK_TABLE.as_ptr().add(index) })
}

/// Squares attacked by a queen on `sq` given the `occupied` squares.
/// Blockers are included, whatever their color.
#[inline]
pub const fn queen_attacks(sq: Square, occupied: Bitboard) -> Bitboard {
    Bitboard(rook_attacks(sq, occupied).0 | bishop_attacks(sq, occupied).0)
}

/// Squares attacked by a `color` `piece` on `sq` given the `occupied`
/// squares. A generic dispatch for exchange evaluation and legality checks;
/// move generation calls the specific functions instead.
#[inline]
pub const fn attacks(piece: PieceType, color: Color, sq: Square, occupied: Bitboard) -> Bitboard {
    match piece {
        PieceType::Pawn => pawn_attacks(color, sq),
        PieceType::Knight => knight_attacks(sq),
        PieceType::Bishop => bishop_attacks(sq, occupied),
        PieceType::Rook => rook_attacks(sq, occupied),
        PieceType::Queen => queen_attacks(sq, occupied),
        PieceType::King => king_attacks(sq),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prng::Prng;
    use crate::types::{File, Rank};

    fn bb(squares: &[Square]) -> Bitboard {
        squares
            .iter()
            .fold(Bitboard::EMPTY, |b, &sq| b | Bitboard::from_square(sq))
    }

    fn total(attacks: fn(Square) -> Bitboard) -> u32 {
        Square::ALL.into_iter().map(|sq| attacks(sq).count()).sum()
    }

    #[test]
    fn knight_counts() {
        assert_eq!(knight_attacks(Square::A1).count(), 2);
        assert_eq!(knight_attacks(Square::B1).count(), 3);
        assert_eq!(knight_attacks(Square::E4).count(), 8);
        assert_eq!(knight_attacks(Square::H8).count(), 2);
        assert_eq!(total(knight_attacks), 336);
    }

    #[test]
    fn king_counts() {
        assert_eq!(king_attacks(Square::A1).count(), 3);
        assert_eq!(king_attacks(Square::E1).count(), 5);
        assert_eq!(king_attacks(Square::E4).count(), 8);
        assert_eq!(total(king_attacks), 420);
    }

    #[test]
    fn pawn_specific() {
        assert_eq!(pawn_attacks(Color::White, Square::A2), bb(&[Square::B3]));
        assert_eq!(
            pawn_attacks(Color::White, Square::E2),
            bb(&[Square::D3, Square::F3])
        );
        assert_eq!(pawn_attacks(Color::White, Square::H2), bb(&[Square::G3]));
        assert_eq!(
            pawn_attacks(Color::Black, Square::E7),
            bb(&[Square::D6, Square::F6])
        );
    }

    #[test]
    fn reciprocity() {
        for a in Square::ALL {
            for b in Square::ALL {
                assert_eq!(
                    knight_attacks(a).contains(b),
                    knight_attacks(b).contains(a),
                    "knight {a} {b}"
                );
                assert_eq!(
                    king_attacks(a).contains(b),
                    king_attacks(b).contains(a),
                    "king {a} {b}"
                );
                assert_eq!(
                    pawn_attacks(Color::White, a).contains(b),
                    pawn_attacks(Color::Black, b).contains(a),
                    "pawn {a} {b}"
                );
            }
        }
    }

    #[test]
    fn pawn_color_symmetry() {
        for sq in Square::ALL {
            assert_eq!(
                pawn_attacks(Color::Black, sq),
                pawn_attacks(Color::White, sq.flip_vertical()).flip_vertical(),
                "{sq}"
            );
        }
    }

    #[test]
    fn pawn_attacks_bb_is_union() {
        let sets = [
            Bitboard::EMPTY,
            Bitboard::FULL,
            Bitboard::from_rank(Rank::R2),
            Bitboard::from_rank(Rank::R7),
            Bitboard::from_file(File::A),
            Bitboard::from_file(File::H),
            bb(&[Square::A2, Square::D4, Square::E5, Square::H7]),
        ];
        for pawns in sets {
            for color in Color::ALL {
                let expected = pawns
                    .into_iter()
                    .fold(Bitboard::EMPTY, |b, sq| b | pawn_attacks(color, sq));
                assert_eq!(pawn_attacks_bb(color, pawns), expected, "{color}");
            }
        }
    }

    #[test]
    fn between_specific() {
        assert_eq!(
            between(Square::A1, Square::H8),
            bb(&[
                Square::B2,
                Square::C3,
                Square::D4,
                Square::E5,
                Square::F6,
                Square::G7,
            ])
        );
        assert_eq!(
            between(Square::A1, Square::A8),
            Bitboard::from_file(File::A).without(bb(&[Square::A1, Square::A8]))
        );
        assert!(between(Square::E4, Square::E5).is_empty());
        assert!(between(Square::A1, Square::B3).is_empty());
    }

    #[test]
    fn line_specific() {
        let diagonal = line(Square::A1, Square::H8);
        assert_eq!(diagonal.count(), 8);
        for i in 0..8 {
            assert!(diagonal.contains(Square::new(File::ALL[i], Rank::ALL[i])));
        }
        assert_eq!(line(Square::E4, Square::E5), Bitboard::from_file(File::E));
        assert!(line(Square::A1, Square::B3).is_empty());
    }

    #[test]
    fn same_square_is_empty() {
        for sq in Square::ALL {
            assert!(between(sq, sq).is_empty(), "{sq}");
            assert!(line(sq, sq).is_empty(), "{sq}");
        }
    }

    #[test]
    fn geometry_exhaustive() {
        for a in Square::ALL {
            for b in Square::ALL {
                let inside = between(a, b);
                let full = line(a, b);
                assert_eq!(inside, between(b, a), "between {a} {b}");
                assert_eq!(full, line(b, a), "line {a} {b}");
                assert!(inside.without(full).is_empty(), "between ⊄ line {a} {b}");
                if !full.is_empty() {
                    assert!(full.contains(a) && full.contains(b), "line {a} {b}");
                }
                for c in inside {
                    assert!(aligned(a, b, c), "{a} {b} {c}");
                }
            }
        }
    }

    fn rook_slow(sq: Square, occupied: Bitboard) -> Bitboard {
        Bitboard(reference::rook_attacks_slow(sq.index(), occupied.0))
    }

    fn bishop_slow(sq: Square, occupied: Bitboard) -> Bitboard {
        Bitboard(reference::bishop_attacks_slow(sq.index(), occupied.0))
    }

    #[test]
    fn slow_empty_board_counts() {
        assert_eq!(rook_slow(Square::A1, Bitboard::EMPTY).count(), 14);
        assert_eq!(rook_slow(Square::E4, Bitboard::EMPTY).count(), 14);
        assert_eq!(bishop_slow(Square::A1, Bitboard::EMPTY).count(), 7);
        assert_eq!(bishop_slow(Square::E4, Bitboard::EMPTY).count(), 13);
    }

    #[test]
    fn slow_rook_stops_on_blockers() {
        let attacks = rook_slow(Square::E4, bb(&[Square::E6, Square::C4]));
        for sq in [Square::E5, Square::E6, Square::D4, Square::C4] {
            assert!(attacks.contains(sq), "{sq}");
        }
        for sq in [Square::E7, Square::E8, Square::B4, Square::A4] {
            assert!(!attacks.contains(sq), "{sq}");
        }
    }

    #[test]
    fn slow_surrounded_sliders() {
        let diagonal_neighbors = bb(&[Square::D3, Square::F3, Square::D5, Square::F5]);
        assert_eq!(
            bishop_slow(Square::E4, diagonal_neighbors),
            diagonal_neighbors
        );
        for sq in Square::ALL {
            let neighbors = king_attacks(sq);
            assert_eq!(
                bishop_slow(sq, neighbors) | rook_slow(sq, neighbors),
                neighbors,
                "{sq}"
            );
        }
    }

    #[test]
    fn slow_never_attacks_own_square() {
        let sets = [
            Bitboard::EMPTY,
            Bitboard::FULL,
            Bitboard::EDGES,
            Bitboard::DARK_SQUARES,
            Bitboard::LIGHT_SQUARES,
            bb(&[Square::A2, Square::D4, Square::E5, Square::H7]),
        ];
        for occupied in sets {
            for sq in Square::ALL {
                let with_self = occupied | Bitboard::from_square(sq);
                for occupied in [occupied, with_self] {
                    assert!(!rook_slow(sq, occupied).contains(sq), "rook {sq}");
                    assert!(!bishop_slow(sq, occupied).contains(sq), "bishop {sq}");
                }
            }
        }
    }

    #[test]
    fn slow_empty_board_matches_lines() {
        for a in Square::ALL {
            let own = Bitboard::from_square(a);
            let rook = rook_slow(a, Bitboard::EMPTY);
            let bishop = bishop_slow(a, Bitboard::EMPTY);
            assert_eq!(
                rook,
                (Bitboard::from_rank(a.rank()) | Bitboard::from_file(a.file())).without(own),
                "{a}"
            );
            assert!((rook & bishop).is_empty(), "{a}");
            let lines = Square::ALL
                .into_iter()
                .fold(Bitboard::EMPTY, |acc, b| acc | line(a, b));
            assert_eq!(rook | bishop, lines.without(own), "{a}");
        }
    }

    type Slider = fn(Square, Bitboard) -> Bitboard;

    /// `(name, entries, magic attacks, reference attacks)` for both sliders.
    const SLIDERS: [(&str, &[magic::MagicEntry; 64], Slider, Slider); 2] = [
        ("rook", &magic::ROOK_ENTRIES, rook_attacks, rook_slow),
        (
            "bishop",
            &magic::BISHOP_ENTRIES,
            bishop_attacks,
            bishop_slow,
        ),
    ];

    /// A mix of sparse and dense random occupancies.
    fn random_occupancies(count: usize) -> Vec<Bitboard> {
        let mut prng = Prng::new(0x1234_5678_9ABC_DEF0);
        (0..count)
            .map(|i| match i % 3 {
                0 => prng.next_u64() & prng.next_u64() & prng.next_u64(),
                1 => prng.next_u64() & prng.next_u64(),
                _ => prng.next_u64(),
            })
            .map(Bitboard)
            .collect()
    }

    #[test]
    fn magic_table_sizes() {
        assert_eq!(magic::ROOK_TABLE.len(), 102_400);
        assert_eq!(magic::BISHOP_TABLE.len(), 5_248);
    }

    #[test]
    fn magic_matches_reference_exhaustive() {
        let mut cases = 0;
        for (name, entries, fast, slow) in SLIDERS {
            for sq in Square::ALL {
                let mask = entries[sq.index()].mask;
                let mut subset = 0u64;
                loop {
                    let occupied = Bitboard(subset);
                    assert_eq!(fast(sq, occupied), slow(sq, occupied), "{name} {sq}");
                    cases += 1;
                    subset = subset.wrapping_sub(mask) & mask;
                    if subset == 0 {
                        break;
                    }
                }
            }
        }
        assert_eq!(cases, 107_648);
    }

    #[test]
    fn magic_ignores_bits_outside_mask() {
        for occupied in random_occupancies(3_000) {
            for (name, entries, fast, slow) in SLIDERS {
                for sq in Square::ALL {
                    let relevant = Bitboard(occupied.0 & entries[sq.index()].mask);
                    let attacks = fast(sq, occupied);
                    assert_eq!(attacks, fast(sq, relevant), "{name} {sq}");
                    assert_eq!(attacks, slow(sq, occupied), "{name} {sq}");
                }
            }
        }
    }

    #[test]
    fn queen_is_rook_and_bishop() {
        for occupied in random_occupancies(300) {
            for sq in Square::ALL {
                assert_eq!(
                    queen_attacks(sq, occupied),
                    rook_attacks(sq, occupied) | bishop_attacks(sq, occupied),
                    "{sq}"
                );
            }
        }
    }

    #[test]
    fn attacks_dispatch() {
        for occupied in random_occupancies(30) {
            for sq in Square::ALL {
                for color in Color::ALL {
                    let of = |piece| attacks(piece, color, sq, occupied);
                    assert_eq!(of(PieceType::Pawn), pawn_attacks(color, sq));
                    assert_eq!(of(PieceType::Knight), knight_attacks(sq));
                    assert_eq!(of(PieceType::Bishop), bishop_attacks(sq, occupied));
                    assert_eq!(of(PieceType::Rook), rook_attacks(sq, occupied));
                    assert_eq!(of(PieceType::Queen), queen_attacks(sq, occupied));
                    assert_eq!(of(PieceType::King), king_attacks(sq));
                }
            }
        }
    }

    // The build script uses a fixed seed: if these change, the tables were
    // regenerated differently (a changed seed, PRNG or search).
    #[test]
    fn magic_search_is_deterministic() {
        assert_eq!(
            magic::ROOK_ENTRIES[Square::A1.index()].magic,
            0x1080_0040_0880_1020
        );
        assert_eq!(
            magic::BISHOP_ENTRIES[Square::A1.index()].magic,
            0xA010_0411_0800_3100
        );
    }
}
