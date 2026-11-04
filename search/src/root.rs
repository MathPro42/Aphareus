use std::thread;
use std::time::Duration;

use engine::movegen::legal_moves;
use engine::{Move, Position};
use eval::Score;

use crate::Limits;
use crate::alphabeta::alphabeta;
use crate::report::{IterationInfo, Reporter};
use crate::shared::Shared;
use crate::thread::ThreadState;
use crate::time::TimeManager;

/// Searches `position` within `limits` by iterative deepening, then reports
/// the best move once. `Move::NULL` if there is no legal move.
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

    let mut list = legal_moves(t.position.board());
    let moves = list.as_mut_slice();
    if moves.is_empty() {
        wait_if_infinite(shared, limits);
        reporter.best_move(Move::NULL);
        return;
    }

    let mut best = moves[0];
    // Depth 0 still runs one iteration: the move comes from a search.
    let max_depth = limits.depth.map_or(MAX_DEPTH, |d| d.clamp(1, MAX_DEPTH));
    for depth in 1..=max_depth {
        let (score, mv) = search_root(t, shared, depth, moves);
        // An interrupted iteration is ignored.
        if t.stopped {
            break;
        }
        best = mv;
        move_to_front(moves, mv);
        reporter.iteration(&IterationInfo {
            depth,
            seldepth: t.seldepth,
            score,
            nodes: t.nodes,
            time_ms: t.time.elapsed_ms(),
            pv: t.pv.line(),
        });
        t.can_stop = true;
        // Every line of `depth` plies has been searched: no shorter mate.
        if score >= Score::mate_in(depth as usize) || t.time.soft_exceeded() {
            break;
        }
    }

    wait_if_infinite(shared, limits);
    reporter.best_move(best);
}

/// The deepest iteration: its leaves are at the last ply searched.
const MAX_DEPTH: u32 = Score::MAX_PLY as u32 - 1;

/// One iteration, with a full window. Returns the best score and move. If
/// the search stops, the result is incomplete and must be ignored.
fn search_root(t: &mut ThreadState, shared: &Shared, depth: u32, moves: &[Move]) -> (Score, Move) {
    let mut alpha = -Score::INFINITE;
    let beta = Score::INFINITE;
    let mut best_score = -Score::INFINITE;
    let mut best = moves[0];
    t.nodes += 1;
    t.pv.clear_ply(0);
    t.check_stop(shared);
    if t.stopped {
        return (best_score, best);
    }
    for &mv in moves {
        t.stack.at(0).mv = mv;
        t.position.make_move(mv);
        let score = -alphabeta(t, shared, depth as i32 - 1, -beta, -alpha, 1);
        t.position.unmake();
        if t.stopped {
            break;
        }
        if score > best_score {
            best_score = score;
            best = mv;
            t.pv.update(0, mv);
            alpha = alpha.max(score);
        }
    }
    (best_score, best)
}

/// Puts `mv` first; the moves before it shift by one, in the same order.
fn move_to_front(moves: &mut [Move], mv: Move) {
    if let Some(i) = moves.iter().position(|&m| m == mv) {
        moves[..=i].rotate_right(1);
    }
}

/// With `go infinite`, UCI forbids `bestmove` before `stop`.
fn wait_if_infinite(shared: &Shared, limits: &Limits) {
    if limits.infinite {
        while !shared.stop_requested() {
            thread::sleep(Duration::from_millis(1));
        }
    }
}

#[cfg(test)]
mod tests {
    use engine::Board;

    use super::*;

    const STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    /// Depth 1 takes the pawn with Rxa7; depth 2 sees Re1# and plays f4.
    const GREEDY: &str = "4r1k1/p4ppp/8/8/8/8/5PPP/R5K1 w - - 0 1";

    /// Every report of one search.
    #[derive(Default)]
    struct Log {
        /// Depth, node count and principal variation of each iteration.
        iterations: Vec<(u32, u64, Vec<Move>)>,
        best: Vec<Move>,
    }

    impl Reporter for Log {
        fn iteration(&mut self, info: &IterationInfo<'_>) {
            self.iterations
                .push((info.depth, info.nodes, info.pv.to_vec()));
        }

        fn best_move(&mut self, mv: Move) {
            self.best.push(mv);
        }
    }

    fn run(fen: &str, limits: &Limits) -> (Box<ThreadState>, Log) {
        let mut t = Box::new(ThreadState::new());
        let mut log = Log::default();
        let position = Position::new(Board::from_fen(fen).unwrap());
        search(&mut t, &Shared::new(), position, limits, &mut log);
        (t, log)
    }

    fn nodes(n: u64) -> Limits {
        Limits {
            nodes: Some(n),
            ..Limits::default()
        }
    }

    #[test]
    fn node_limit_is_exact() {
        let (_, full) = run(STARTPOS, &Limits::depth(2));
        let (first, second) = (full.iterations[0].1, full.iterations[1].1);
        // One node past the end of an iteration: the limit is hit on the
        // root node of the next one.
        for n in [first + 1, second + 1, 5000] {
            let (t, _) = run(STARTPOS, &nodes(n));
            assert_eq!(t.nodes, n);
        }
    }

    #[test]
    fn first_iteration_ignores_the_node_limit() {
        let (t, log) = run(STARTPOS, &nodes(1));
        assert_eq!(log.iterations.len(), 1);
        assert!(t.nodes > 1);
        assert_eq!(log.best, [log.iterations[0].2[0]]);
    }

    #[test]
    fn interrupted_iteration_is_ignored() {
        let (_, full) = run(GREEDY, &Limits::depth(2));
        let (_, first_nodes, ref first_pv) = full.iterations[0];
        let (_, second_nodes, ref second_pv) = full.iterations[1];
        // The second iteration changes its mind.
        assert_ne!(first_pv[0], second_pv[0]);

        // Stopped anywhere in the second iteration, even on its last node.
        for n in first_nodes + 1..=second_nodes {
            let (_, log) = run(GREEDY, &nodes(n));
            assert_eq!(log.iterations.len(), 1, "{n}");
            assert_eq!(log.best, [first_pv[0]], "{n}");
        }
        let (_, log) = run(GREEDY, &nodes(second_nodes + 1));
        assert_eq!(log.iterations.len(), 2);
        assert_eq!(log.best, [second_pv[0]]);
    }

    #[test]
    fn best_move_goes_first_and_others_keep_their_order() {
        let list = legal_moves(&Board::startpos());
        let before = list.as_slice().to_vec();
        for &mv in &before {
            let mut moves = before.clone();
            move_to_front(&mut moves, mv);
            let mut expected = vec![mv];
            expected.extend(before.iter().filter(|&&m| m != mv));
            assert_eq!(moves, expected);
        }
    }
}
