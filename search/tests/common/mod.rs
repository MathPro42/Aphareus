use std::sync::{Arc, Mutex};

use engine::{Board, Position, from_uci};
use search::{Limits, RecordingReporter, Reporter, SearchPool};

/// `fen`, then `moves` played as in `position fen … moves …`.
pub fn position(fen: &str, moves: &str) -> Position {
    let board = Board::from_fen(fen).unwrap_or_else(|e| panic!("{fen}: {e}"));
    let mut position = Position::new(board);
    for text in moves.split_whitespace() {
        let mv = from_uci(position.board(), text, false).unwrap_or_else(|| panic!("{text}"));
        position.make_move(mv);
    }
    position
}

/// Runs one search through a new pool and returns its reporter.
pub fn search_with<R: Reporter + 'static>(position: Position, limits: Limits, reporter: R) -> R {
    let shared = Arc::new(Mutex::new(reporter));
    let mut pool = SearchPool::new();
    pool.start(position, limits, Box::new(Arc::clone(&shared)));
    pool.wait();
    // After `wait`, the search thread has released its clone.
    Arc::into_inner(shared).unwrap().into_inner().unwrap()
}

pub fn search(position: Position, limits: Limits) -> RecordingReporter {
    search_with(position, limits, RecordingReporter::default())
}
