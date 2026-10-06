mod common;

use common::{CHESS960, STANDARD, load};
use engine::movegen::legal_moves;
use engine::{Board, CastleSide, Move, PieceType, Position, Square, from_uci, to_uci};

use Square::*;

fn parse(fen: &str) -> Board {
    Board::from_fen(fen).unwrap_or_else(|e| panic!("{fen}: {e}"))
}

/// Plays moves given in standard UCI notation.
fn play(position: &mut Position, moves: &[&str]) {
    for &text in moves {
        let m = from_uci(position.board(), text, false)
            .unwrap_or_else(|| panic!("{text} is not legal in {}", position.board().to_fen()));
        position.make_move(m);
    }
}

const CASTLES: &str = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1";

// UCI notation

#[test]
fn uci_double_push() {
    let board = Board::startpos();
    let m = Move::double_push(E2, E4);
    assert_eq!(from_uci(&board, "e2e4", false), Some(m));
    assert_eq!(to_uci(&board, m, false), "e2e4");
}

#[test]
fn uci_promotions() {
    let board = parse("8/4P3/8/8/8/8/8/k3K3 w - - 0 1");
    let queen = Move::promotion(E7, E8, PieceType::Queen, false);
    assert_eq!(from_uci(&board, "e7e8q", false), Some(queen));
    assert_eq!(to_uci(&board, queen, false), "e7e8q");
    assert_eq!(
        from_uci(&board, "e7e8n", false),
        Some(Move::promotion(E7, E8, PieceType::Knight, false))
    );
    assert_eq!(from_uci(&board, "e7e8", false), None);
}

#[test]
fn uci_castles_standard() {
    let board = parse(CASTLES);
    let short = Move::castle(E1, H1, CastleSide::King);
    let long = Move::castle(E1, A1, CastleSide::Queen);
    assert_eq!(to_uci(&board, short, false), "e1g1");
    assert_eq!(to_uci(&board, long, false), "e1c1");
    assert_eq!(from_uci(&board, "e1g1", false), Some(short));
    assert_eq!(from_uci(&board, "e1h1", false), Some(short));
    assert_eq!(from_uci(&board, "e1c1", false), Some(long));
    assert_eq!(from_uci(&board, "e1a1", false), Some(long));

    let black = parse("r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R b KQkq - 0 1");
    let black_short = Move::castle(E8, H8, CastleSide::King);
    assert_eq!(to_uci(&black, black_short, false), "e8g8");
    assert_eq!(from_uci(&black, "e8g8", false), Some(black_short));
}

#[test]
fn uci_castles_chess960() {
    let board = parse(CASTLES);
    let short = Move::castle(E1, H1, CastleSide::King);
    assert_eq!(to_uci(&board, short, true), "e1h1");
    assert_eq!(from_uci(&board, "e1h1", true), Some(short));
    assert_eq!(from_uci(&board, "e1g1", true), None);
}

#[test]
fn uci_rejects() {
    let board = Board::startpos();
    for text in ["e2e5", "zz", "e2", "", "e2e4q", "e2e4 ", "0000"] {
        assert_eq!(from_uci(&board, text, false), None, "{text:?}");
    }
    assert_eq!(to_uci(&board, Move::NULL, false), "0000");
}

/// `from_uci(to_uci(m)) == m` for every legal move of `board` and of each
/// position one move later.
fn check_round_trip(board: &Board, chess960: bool) {
    let mut boards = vec![board.clone()];
    boards.extend(
        legal_moves(board)
            .as_slice()
            .iter()
            .map(|&m| board.make_move(m)),
    );
    for board in &boards {
        for &m in legal_moves(board).as_slice() {
            let text = to_uci(board, m, chess960);
            assert_eq!(
                from_uci(board, &text, chess960),
                Some(m),
                "{} {text}",
                board.to_fen()
            );
        }
    }
}

#[test]
fn uci_round_trip() {
    for case in load(STANDARD) {
        check_round_trip(&case.board, false);
        check_round_trip(&case.board, true);
    }

    for case in load(CHESS960).iter().take(50) {
        check_round_trip(&case.board, true);
    }
}

// Repetitions

#[test]
fn knights_back_and_forth_repeat() {
    let mut position = Position::new(Board::startpos());
    play(&mut position, &["g1f3", "g8f6", "f3g1"]);
    assert_eq!(position.repetitions(), 0);
    play(&mut position, &["f6g8"]);
    assert_eq!(position.board().hash(), Board::startpos().hash());
    assert_eq!(position.repetitions(), 1);
    play(&mut position, &["g1f3", "g8f6", "f3g1", "f6g8"]);
    assert_eq!(position.repetitions(), 2);
}

#[test]
fn a_pawn_move_ends_the_repetitions() {
    let mut position = Position::new(Board::startpos());
    play(&mut position, &["g1f3", "g8f6", "f3g1", "f6g8", "e2e4"]);
    assert_eq!(position.repetitions(), 0);
    play(&mut position, &["e7e5", "g1f3", "g8f6", "f3g1", "f6g8"]);
    assert_eq!(position.repetitions(), 1);
}

#[test]
fn lost_castling_rights_are_not_a_repetition() {
    let mut position = Position::new(parse(CASTLES));
    play(&mut position, &["a1b1", "a8b8", "b1a1", "b8a8"]);
    assert_eq!(position.repetitions(), 0);
    play(&mut position, &["a1b1", "a8b8", "b1a1", "b8a8"]);
    assert_eq!(position.repetitions(), 1);
}

// Other draws

#[test]
fn fifty_move_rule() {
    let position = Position::new(parse("4k3/8/8/8/8/8/4P3/R3K3 w - - 99 80"));
    assert!(!position.is_fifty_move_draw());

    let mut quiet = position.clone();
    play(&mut quiet, &["a1a2"]);
    assert!(quiet.is_fifty_move_draw());

    let mut pawn = position.clone();
    play(&mut pawn, &["e2e3"]);
    assert!(!pawn.is_fifty_move_draw());
}

#[test]
fn insufficient_material() {
    assert!(Position::new(parse("4k3/8/8/8/8/8/8/4K2B w - - 0 1")).is_insufficient_material());
    assert!(!Position::new(Board::startpos()).is_insufficient_material());
}

// Undoing moves

#[test]
fn unmake_restores_the_previous_board() {
    let start = parse("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    let mut position = Position::new(start.clone());
    let mut history = vec![start];
    for i in 0..12 {
        let board = position.board().clone();
        if i % 4 == 3 && !board.in_check() {
            position.make_null_move();
        } else {
            let moves = legal_moves(&board);
            position.make_move(moves.as_slice()[(7 * i) % moves.len()]);
        }
        history.push(position.board().clone());
    }
    while history.len() > 1 {
        history.pop();
        position.unmake();
        assert_eq!(position.board(), history.last().unwrap());
    }
    assert_eq!(position.repetitions(), 0);
}
