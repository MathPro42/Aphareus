use crate::board::Board;
use crate::movegen::{All, Context, generate, legal_moves};
use crate::moves::{Move, MoveList};

/// Number of leaf positions `depth` plies below `board`.
pub fn perft(board: &Board, depth: u32) -> u64 {
    if depth == 0 {
        return 1;
    }
    let mut moves = MoveList::new();
    generate::<All>(&Context::new(board), &mut moves);
    if depth == 1 {
        return moves.len() as u64;
    }
    moves
        .as_slice()
        .iter()
        .map(|&m| perft(&board.make_move(m), depth - 1))
        .sum()
}

#[must_use]
pub fn divide(board: &Board, depth: u32) -> Vec<(Move, u64)> {
    assert!(depth > 0, "divide needs a depth of at least 1");
    legal_moves(board)
        .as_slice()
        .iter()
        .map(|&m| (m, perft(&board.make_move(m), depth - 1)))
        .collect()
}
