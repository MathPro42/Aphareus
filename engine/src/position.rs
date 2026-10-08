use crate::board::Board;
use crate::moves::Move;

const RESERVED: usize = 1024;

#[derive(Clone, Debug)]
pub struct Position {
    boards: Vec<Board>,
    hashes: Vec<u64>,
    /// Plies since the last null move, or since the start, for each board.
    /// No repetition can reach across a null move: only a search plays it.
    plies_from_null: Vec<u16>,
}

impl Position {
    /// A game starting from `board`.
    pub fn new(board: Board) -> Position {
        let mut position = Position {
            boards: Vec::with_capacity(RESERVED),
            hashes: Vec::with_capacity(RESERVED),
            plies_from_null: Vec::with_capacity(RESERVED),
        };
        position.push(board, 0);
        position
    }

    /// The current board.
    #[inline]
    pub fn board(&self) -> &Board {
        self.boards.last().expect("a position always has a board")
    }

    /// Plays `m`, which must be legal on the current board.
    pub fn make_move(&mut self, m: Move) {
        let next = self.board().make_move(m);
        let plies = self.plies_from_null().saturating_add(1);
        self.push(next, plies);
    }

    /// Passes the turn. The side to move must not be in check.
    pub fn make_null_move(&mut self) {
        let next = self.board().make_null_move();
        self.push(next, 0);
    }

    /// Undoes the last move.
    pub fn unmake(&mut self) {
        debug_assert!(self.boards.len() > 1, "unmake: no move to undo");
        if self.boards.len() > 1 {
            self.boards.pop();
            self.hashes.pop();
            self.plies_from_null.pop();
        }
    }

    fn push(&mut self, board: Board, plies_from_null: u16) {
        self.hashes.push(board.hash());
        self.boards.push(board);
        self.plies_from_null.push(plies_from_null);
    }

    fn plies_from_null(&self) -> u16 {
        *self
            .plies_from_null
            .last()
            .expect("a position always has a board")
    }

    #[must_use]
    pub fn repetitions(&self) -> u32 {
        let end = self.hashes.len() - 1;
        let current = self.hashes[end];
        // Neither a capture or pawn move nor a null move can be crossed;
        // `plies_from_null` never exceeds `end`.
        let limit =
            usize::from(self.board().halfmove_clock()).min(usize::from(self.plies_from_null()));
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
