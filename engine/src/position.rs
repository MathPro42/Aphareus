use crate::board::Board;
use crate::moves::Move;

const RESERVED: usize = 1024;

#[derive(Clone, Debug)]
pub struct Position {
    boards: Vec<Board>,
    hashes: Vec<u64>,
}

impl Position {
    /// A game starting from `board`.
    pub fn new(board: Board) -> Position {
        let mut boards = Vec::with_capacity(RESERVED);
        let mut hashes = Vec::with_capacity(RESERVED);
        hashes.push(board.hash());
        boards.push(board);
        Position { boards, hashes }
    }

    /// The current board.
    #[inline]
    pub fn board(&self) -> &Board {
        self.boards.last().expect("a position always has a board")
    }

    /// Plays `m`, which must be legal on the current board.
    pub fn make_move(&mut self, m: Move) {
        let next = self.board().make_move(m);
        self.push(next);
    }

    /// Passes the turn. The side to move must not be in check.
    pub fn make_null_move(&mut self) {
        let next = self.board().make_null_move();
        self.push(next);
    }

    /// Undoes the last move.
    pub fn unmake(&mut self) {
        debug_assert!(self.boards.len() > 1, "unmake: no move to undo");
        self.boards.pop();
        self.hashes.pop();
    }

    fn push(&mut self, board: Board) {
        self.hashes.push(board.hash());
        self.boards.push(board);
    }

    #[must_use]
    pub fn repetitions(&self) -> u32 {
        let end = self.hashes.len() - 1;
        let current = self.hashes[end];
        let limit = usize::from(self.board().halfmove_clock()).min(end);
        let mut count = 0;
        let mut i = 4;
        while i <= limit {
            if self.hashes[end - i] == current {
                count += 1;
            }
            i += 2;
        }
        count
    }

    /// Returns `true` if 100 plies have passed without a capture or pawn
    /// move.
    #[must_use]
    pub fn is_fifty_move_draw(&self) -> bool {
        self.board().halfmove_clock() >= 100
    }

    /// Returns `true` if neither side can ever mate.
    #[must_use]
    pub fn is_insufficient_material(&self) -> bool {
        self.board().is_insufficient_material()
    }
}
