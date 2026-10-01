mod common;

use std::collections::HashSet;

use engine::movegen::{All, Context, GenMode, Noisy, Quiet, generate, legal_moves};
use engine::{Board, CastleSide, File, Move, MoveList, Square};

use Square::*;

fn parse(fen: &str) -> Board {
    Board::from_fen(fen).unwrap_or_else(|e| panic!("{fen}: {e}"))
}

fn moves_of<M: GenMode>(board: &Board) -> MoveList {
    let mut list = MoveList::new();
    generate::<M>(&Context::new(board), &mut list);
    list
}

fn from(list: &MoveList, sq: Square) -> Vec<Move> {
    list.as_slice()
        .iter()
        .copied()
        .filter(|m| m.from() == sq)
        .collect()
}

#[test]
fn move_counts() {
    assert_eq!(legal_moves(&Board::startpos()).len(), 20);
    let kiwipete = parse("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    assert_eq!(legal_moves(&kiwipete).len(), 48);
}

#[test]
fn double_check_only_king_moves() {
    let board = parse("4k3/8/8/8/1b6/3n4/7R/4K3 w - - 0 1");
    let moves = legal_moves(&board);
    assert!(!moves.is_empty());
    assert!(moves.as_slice().iter().all(|m| m.from() == E1), "{moves:?}");
}

#[test]
fn king_does_not_step_back_along_the_checking_line() {
    let board = parse("4k3/4r3/8/8/8/8/8/4K3 w - - 0 1");
    let moves = legal_moves(&board);
    assert!(!moves.contains(Move::quiet(E1, E2)));
    assert_eq!(moves.len(), 4, "{moves:?}");
}

#[test]
fn pinned_knight_never_moves() {
    let board = parse("4k3/4r3/8/8/8/8/4N3/4K3 w - - 0 1");
    assert!(from(&legal_moves(&board), E2).is_empty());
}

#[test]
fn rook_pinned_on_a_file_stays_on_it() {
    let board = parse("4k3/4r3/8/8/8/8/4R3/4K3 w - - 0 1");
    let rook_moves = from(&legal_moves(&board), E2);
    assert_eq!(rook_moves.len(), 5, "{rook_moves:?}");
    assert!(rook_moves.iter().all(|m| m.to().file() == File::E));
    assert!(rook_moves.contains(&Move::capture(E2, E7)));
}

#[test]
fn en_passant_with_horizontal_pin_is_not_generated() {
    let board = parse("8/8/8/KPp4r/8/8/8/7k w - c6 0 2");
    assert_eq!(board.en_passant(), None);
    assert!(!legal_moves(&board).contains(Move::en_passant(B5, C6)));
}

#[test]
fn en_passant_answering_a_check_is_generated() {
    let board = parse("8/8/8/2k5/3Pp3/8/8/4K3 b - d3 0 1");
    assert!(board.in_check());
    assert!(legal_moves(&board).contains(Move::en_passant(E4, D3)));
}

#[test]
fn en_passant_with_diagonal_pin_is_not_generated() {
    let board = parse("7b/8/8/3pP3/8/8/8/K3k3 w - d6 0 2");
    assert_eq!(board.en_passant(), None);
    assert!(!legal_moves(&board).contains(Move::en_passant(E5, D6)));

    let board = parse("7b/8/8/2PpP3/8/8/8/K3k3 w - d6 0 2");
    assert_eq!(board.en_passant(), Some(D6));
    let moves = legal_moves(&board);
    assert!(moves.contains(Move::en_passant(C5, D6)));
    assert!(!moves.contains(Move::en_passant(E5, D6)));
}

fn check_en_passant_square(board: &Board, depth: u32) {
    let moves = legal_moves(board);
    let can_capture = moves.as_slice().iter().any(|m| m.is_en_passant());
    assert_eq!(
        board.en_passant().is_some(),
        can_capture,
        "{}",
        board.to_fen()
    );
    if depth > 0 {
        for &m in moves.as_slice() {
            check_en_passant_square(&board.make_move(m), depth - 1);
        }
    }
}

#[test]
fn en_passant_square_only_with_a_legal_capture() {
    for case in common::load(common::STANDARD) {
        check_en_passant_square(&case.board, 3);
    }
    for case in common::load(common::CHESS960).iter().take(20) {
        check_en_passant_square(&case.board, 2);
    }
    for fen in [
        "8/3p4/8/K3P2r/8/8/8/7k b - - 0 1",
        "7b/3p4/8/4P3/8/8/8/K3k3 b - - 0 1",
        "7b/3p4/8/2P1P3/8/8/8/K3k3 b - - 0 1",
        "4k3/8/8/8/4p3/8/5P2/4RK2 w - - 0 1",
        "8/8/8/2k5/4p3/8/3P4/4K3 w - - 0 1",
    ] {
        check_en_passant_square(&parse(fen), 4);
    }
}

#[test]
fn castle_through_an_attacked_square_is_not_generated() {
    let board = parse("4k3/8/8/8/8/8/5r2/R3K2R w KQ - 0 1");
    let moves = legal_moves(&board);
    assert!(!moves.contains(Move::castle(E1, H1, CastleSide::King)));
    assert!(moves.contains(Move::castle(E1, A1, CastleSide::Queen)));
}

#[test]
fn castle_in_check_is_not_generated() {
    let board = parse("4k3/4r3/8/8/8/8/8/R3K2R w KQ - 0 1");
    assert!(
        legal_moves(&board)
            .as_slice()
            .iter()
            .all(|m| !m.is_castle())
    );
}

#[test]
fn chess960_castle_rook_hiding_an_attack() {
    let board = parse("4k3/8/8/8/8/8/8/rR3K2 w Q - 0 1");
    assert!(!board.in_check());
    assert!(
        legal_moves(&board)
            .as_slice()
            .iter()
            .all(|m| !m.is_castle())
    );
    let board = parse("4k3/8/8/8/8/8/8/1R3K2 w Q - 0 1");
    assert!(legal_moves(&board).contains(Move::castle(F1, B1, CastleSide::Queen)));
}

#[test]
fn modes_partition_the_moves() {
    for case in common::all_cases() {
        let mut boards = vec![case.board.clone()];
        boards.extend(
            legal_moves(&case.board)
                .as_slice()
                .iter()
                .map(|&m| case.board.make_move(m)),
        );
        for board in boards {
            let all: HashSet<Move> = moves_of::<All>(&board).as_slice().iter().copied().collect();
            let noisy = moves_of::<Noisy>(&board);
            let quiet = moves_of::<Quiet>(&board);
            let fen = board.to_fen();
            assert!(noisy.as_slice().iter().all(|m| m.is_noisy()), "{fen}");
            assert!(quiet.as_slice().iter().all(|m| !m.is_noisy()), "{fen}");
            let union: HashSet<Move> = noisy
                .as_slice()
                .iter()
                .chain(quiet.as_slice())
                .copied()
                .collect();
            assert_eq!(noisy.len() + quiet.len(), all.len(), "{fen}");
            assert_eq!(union, all, "{fen}");
        }
    }
}
