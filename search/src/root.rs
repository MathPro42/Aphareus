use std::thread;
use std::time::Duration;

use engine::movegen::legal_moves;
use engine::{Move, Position};

use crate::Limits;
use crate::report::Reporter;
use crate::shared::Shared;
use crate::thread::ThreadState;
use crate::time::TimeManager;

/// Searches `position` within `limits`, then reports the best move once.
/// `Move::NULL` if there is no legal move.
pub(crate) fn search(
    t: &mut ThreadState,
    shared: &Shared,
    position: Position,
    limits: &Limits,
    reporter: &mut dyn Reporter,
) {
    t.position = position;
    t.nodes = 0;
    t.seldepth = 0;
    t.stopped = false;
    t.time = TimeManager::new(limits, t.position.board().side_to_move());
    t.node_limit = limits.nodes;
    t.can_stop = false;

    let moves = legal_moves(t.position.board());
    let best = moves.as_slice().first().copied().unwrap_or(Move::NULL);

    wait_if_infinite(shared, limits);
    reporter.best_move(best);
}

/// With `go infinite`, UCI forbids `bestmove` before `stop`.
fn wait_if_infinite(shared: &Shared, limits: &Limits) {
    if limits.infinite {
        while !shared.stop_requested() {
            thread::sleep(Duration::from_millis(1));
        }
    }
}
