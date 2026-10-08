use engine::{Bitboard, Board, CastleSide, Color, Move, PieceType, Square};

use CastleSide::{King, Queen};
use Square::*;

fn parse(fen: &str) -> Board {
    Board::from_fen(fen).unwrap_or_else(|e| panic!("{fen}: {e}"))
}

/// Plays `m` from `before` and checks the result against the board read
/// from `after`.
fn check(before: &str, m: Move, after: &str) {
    let _ = play(before, m, after);
}

/// [`check`], returning the board reached for further assertions.
fn play(before: &str, m: Move, after: &str) -> Board {
    let made = parse(before).make_move(m);
    let expected = parse(after);
    assert_same(&made, &expected, &format!("{before} + {m:?}"));
    made
}

fn assert_same(made: &Board, expected: &Board, context: &str) {
    assert_eq!(made.to_fen(), expected.to_fen(), "{context}");
    for color in Color::ALL {
        assert_eq!(made.color_bb(color), expected.color_bb(color), "{context}");
        assert_eq!(
            made.non_pawn_hash(color),
            expected.non_pawn_hash(color),
            "{context}"
        );
    }
    for piece in PieceType::ALL {
        assert_eq!(made.pieces(piece), expected.pieces(piece), "{context}");
    }
    for sq in Square::ALL {
        assert_eq!(made.piece_on(sq), expected.piece_on(sq), "{context} {sq}");
    }
    assert_eq!(made.side_to_move(), expected.side_to_move(), "{context}");
    assert_eq!(
        made.castling_rights(),
        expected.castling_rights(),
        "{context}"
    );
    assert_eq!(made.en_passant(), expected.en_passant(), "{context}");
    assert_eq!(
        made.halfmove_clock(),
        expected.halfmove_clock(),
        "{context}"
    );
    assert_eq!(
        made.fullmove_number(),
        expected.fullmove_number(),
        "{context}"
    );
    assert_eq!(made.hash(), expected.hash(), "{context}");
    assert_eq!(made.pawn_hash(), expected.pawn_hash(), "{context}");
    assert_eq!(made.checkers(), expected.checkers(), "{context}");
    assert_eq!(
        made.compute_pinned(),
        expected.compute_pinned(),
        "{context}"
    );
}

const CASTLES: &str = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1";
const CASTLES_BLACK: &str = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R b KQkq - 0 1";

// Pawn and piece moves

#[test]
fn quiet_move() {
    check(
        Board::STARTPOS_FEN,
        Move::quiet(G1, F3),
        "rnbqkbnr/pppppppp/8/8/8/5N2/PPPPPPPP/RNBQKB1R b KQkq - 1 1",
    );
}

#[test]
fn double_push_without_en_passant() {
    check(
        Board::STARTPOS_FEN,
        Move::double_push(E2, E4),
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1",
    );
}

#[test]
fn double_push_with_en_passant() {
    let board = play(
        "rnbqkbnr/ppp1pppp/8/8/3p4/8/PPPPPPPP/RNBQKBNR w KQkq - 0 3",
        Move::double_push(E2, E4),
        "rnbqkbnr/ppp1pppp/8/8/3pP3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 3",
    );
    assert_eq!(board.en_passant(), Some(E3));
}

#[test]
fn capture() {
    check(
        "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",
        Move::capture(E4, D5),
        "rnbqkbnr/ppp1pppp/8/3P4/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 2",
    );
}

#[test]
fn en_passant_capture() {
    check(
        "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3",
        Move::en_passant(E5, F6),
        "rnbqkbnr/ppp1p1pp/5P2/3p4/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 3",
    );
}

#[test]
fn promotions() {
    for (piece, letter) in [
        (PieceType::Knight, 'N'),
        (PieceType::Bishop, 'B'),
        (PieceType::Rook, 'R'),
        (PieceType::Queen, 'Q'),
    ] {
        check(
            "8/P3k3/8/8/8/8/8/4K3 w - - 5 40",
            Move::promotion(A7, A8, piece, false),
            &format!("{letter}7/4k3/8/8/8/8/8/4K3 b - - 0 40"),
        );
    }
}

#[test]
fn promotion_with_capture() {
    check(
        "1r6/P3k3/8/8/8/8/8/4K3 w - - 0 1",
        Move::promotion(A7, B8, PieceType::Queen, true),
        "1Q6/4k3/8/8/8/8/8/4K3 b - - 0 1",
    );
}

// Castles

#[test]
fn white_castles() {
    check(
        CASTLES,
        Move::castle(E1, H1, King),
        "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R4RK1 b kq - 1 1",
    );
    check(
        CASTLES,
        Move::castle(E1, A1, Queen),
        "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/2KR3R b kq - 1 1",
    );
}

#[test]
fn black_castles() {
    check(
        CASTLES_BLACK,
        Move::castle(E8, H8, King),
        "r4rk1/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQ - 1 2",
    );
    check(
        CASTLES_BLACK,
        Move::castle(E8, A8, Queen),
        "2kr3r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQ - 1 2",
    );
}

#[test]
fn chess960_castle_king_does_not_move() {
    check(
        "4k3/8/8/8/8/8/8/6KR w K - 0 1",
        Move::castle(G1, H1, King),
        "4k3/8/8/8/8/8/8/5RK1 b - - 1 1",
    );
}

#[test]
fn chess960_castle_rook_lands_on_king_start() {
    // King f1 goes to g1, rook h1 to f1.
    check(
        "4k3/8/8/8/8/8/8/5K1R w K - 0 1",
        Move::castle(F1, H1, King),
        "4k3/8/8/8/8/8/8/5RK1 b - - 1 1",
    );
    // King d1 goes to c1, rook a1 to d1.
    check(
        "4k3/8/8/8/8/8/8/R2K4 w Q - 0 1",
        Move::castle(D1, A1, Queen),
        "4k3/8/8/8/8/8/8/2KR4 b - - 1 1",
    );
}

#[test]
fn chess960_castle_king_jumps_over_rook() {
    // King g1, queen side rook f1: the king lands on c1, the rook on d1.
    check(
        "4k3/8/8/8/8/8/8/5RK1 w Q - 0 1",
        Move::castle(G1, F1, Queen),
        "4k3/8/8/8/8/8/8/2KR4 b - - 1 1",
    );
}

// Castling rights

#[test]
fn king_move_drops_both_rights() {
    check(
        CASTLES,
        Move::quiet(E1, F1),
        "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R4K1R b kq - 1 1",
    );
}

#[test]
fn rook_move_drops_one_right() {
    check(
        CASTLES,
        Move::quiet(A1, B1),
        "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/1R2K2R b Kkq - 1 1",
    );
}

#[test]
fn rook_captured_in_its_corner() {
    check(
        "r3k2r/8/8/8/8/8/6B1/R3K2R w KQkq - 0 1",
        Move::capture(G2, A8),
        "B3k2r/8/8/8/8/8/8/R3K2R b KQk - 0 1",
    );
    // Both sides lose a right, and the rook gives check.
    let board = play(
        "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
        Move::capture(H1, H8),
        "r3k2R/8/8/8/8/8/8/R3K3 b Qq - 0 1",
    );
    assert_eq!(board.checkers(), Bitboard::from_square(H8));
}

// Counters

#[test]
fn halfmove_clock_resets() {
    // Pawn move.
    check(
        "4k3/8/8/8/8/8/4P3/4K3 w - - 10 30",
        Move::quiet(E2, E3),
        "4k3/8/8/8/8/4P3/8/4K3 b - - 0 30",
    );
    // Capture, which also gives check.
    let board = play(
        "4k3/8/5p2/8/4N3/8/8/4K3 w - - 7 30",
        Move::capture(E4, F6),
        "4k3/8/5N2/8/8/8/8/4K3 b - - 0 30",
    );
    assert!(board.in_check());
}

#[test]
fn fullmove_number_after_black() {
    check(
        "4k3/8/8/8/8/8/8/4K3 b - - 3 41",
        Move::quiet(E8, D8),
        "3k4/8/8/8/8/8/8/4K3 w - - 4 42",
    );
}

#[test]
fn fullmove_number_saturates() {
    check(
        "4k3/8/8/8/8/8/8/4K3 b - - 3 65535",
        Move::quiet(E8, D8),
        "3k4/8/8/8/8/8/8/4K3 w - - 4 65535",
    );
}

// Checks and pins

#[test]
fn move_giving_check() {
    let board = play(
        "4k3/8/8/8/8/8/8/R3K3 w - - 0 1",
        Move::quiet(A1, A8),
        "R3k3/8/8/8/8/8/8/4K3 b - - 1 1",
    );
    assert_eq!(board.checkers(), Bitboard::from_square(A8));
}

#[test]
fn move_creating_a_pin() {
    let board = play(
        "3k4/8/3n4/8/8/8/8/R3K3 w - - 0 1",
        Move::quiet(A1, D1),
        "3k4/8/3n4/8/8/8/8/3RK3 b - - 1 1",
    );
    assert_eq!(board.compute_pinned(), Bitboard::from_square(D6));
}

// Null move

#[test]
fn null_move() {
    let before = parse("rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3");
    let after = before.make_null_move();
    let expected = parse("rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR b KQkq - 1 3");
    assert_same(&after, &expected, "null move");
    assert_eq!(after, expected);
}

#[test]
fn null_move_keeps_pins_of_the_new_side() {
    // After the null move, Black's knight on e7 is pinned by the rook.
    let after = parse("4k3/4n3/8/8/8/8/8/K3R3 w - - 0 1").make_null_move();
    assert_eq!(after.side_to_move(), Color::Black);
    assert_eq!(after.compute_pinned(), Bitboard::from_square(E7));
    assert!(after.checkers().is_empty());
}

#[test]
#[should_panic(expected = "null move while in check")]
fn null_move_in_check_panics() {
    let _ = parse("4k3/8/8/8/8/8/4r3/4K3 w - - 0 1").make_null_move();
}

// Moves that do not fit the board

#[test]
#[should_panic(expected = "e2 is taken")]
fn move_onto_an_occupied_square_panics() {
    let _ = Board::startpos().make_move(Move::quiet(G1, E2));
}

// Transpositions

#[test]
fn en_passant_transpositions() {
    // (before the double push, the push, before the single push, the push,
    // whether the en passant capture is legal)
    let cases = [
        // exd6 would expose the king on a5 to the rook on h5.
        (
            "8/3p4/8/K3P2r/8/8/8/7k b - - 0 1",
            Move::double_push(D7, D5),
            "8/8/3p4/K3P2r/8/8/8/7k b - - 0 1",
            Move::quiet(D6, D5),
            false,
        ),
        // The e5 pawn is pinned on a1-h8 by the bishop.
        (
            "7b/3p4/8/4P3/8/8/8/K3k3 b - - 0 1",
            Move::double_push(D7, D5),
            "7b/8/3p4/4P3/8/8/8/K3k3 b - - 0 1",
            Move::quiet(D6, D5),
            false,
        ),
        // The e4 pawn is pinned on the e-file by the rook.
        (
            "4k3/8/8/8/4p3/8/5P2/4RK2 w - - 0 1",
            Move::double_push(F2, F4),
            "4k3/8/8/8/4p3/5P2/8/4RK2 w - - 0 1",
            Move::quiet(F3, F4),
            false,
        ),
        // Without the rook, exd6 is legal: the positions really differ.
        (
            "8/3p4/8/K3P3/8/8/8/7k b - - 0 1",
            Move::double_push(D7, D5),
            "8/8/3p4/K3P3/8/8/8/7k b - - 0 1",
            Move::quiet(D6, D5),
            true,
        ),
    ];
    for (before_double, double, before_single, single, legal) in cases {
        let by_double = parse(before_double).make_move(double);
        let by_single = parse(before_single).make_move(single);
        let context = format!("{before_double} + {double:?}");
        assert_eq!(by_double.en_passant().is_some(), legal, "{context}");
        assert_eq!(by_single.en_passant(), None, "{context}");
        if legal {
            assert_ne!(by_double.hash(), by_single.hash(), "{context}");
        } else {
            assert_same(&by_double, &by_single, &context);
            assert_eq!(by_double, by_single, "{context}");
        }
    }
}
