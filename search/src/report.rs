use std::sync::{Arc, Mutex, PoisonError};

use engine::Move;
use eval::Score;

/// What one complete iteration found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IterationInfo<'a> {
    pub depth: u32,
    /// Highest ply reached, quiescence included.
    pub seldepth: u32,
    /// From the point of view of the side to move at the root.
    pub score: Score,
    pub nodes: u64,
    pub time_ms: u64,
    /// Principal variation, starting at the root.
    pub pv: &'a [Move],
}

/// Receives the results of a search. The search never writes any output:
/// the caller decides what to do with them.
pub trait Reporter: Send {
    /// Called at the end of each complete iteration.
    fn iteration(&mut self, info: &IterationInfo<'_>);

    /// Called once, at the very end. `Move::NULL` means no legal move
    /// (mate or stalemate at the root).
    fn best_move(&mut self, mv: Move);
}

/// Ignores everything: bench.
#[derive(Clone, Copy, Debug, Default)]
pub struct SilentReporter;

impl Reporter for SilentReporter {
    fn iteration(&mut self, _info: &IterationInfo<'_>) {}

    fn best_move(&mut self, _mv: Move) {}
}

/// Keeps every report, in order: tests.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RecordingReporter {
    /// Depth, score and principal variation of each iteration.
    pub iterations: Vec<(u32, Score, Vec<Move>)>,
    /// Every `best_move` call, so tests can check there is exactly one.
    pub best_moves: Vec<Move>,
}

impl Reporter for RecordingReporter {
    fn iteration(&mut self, info: &IterationInfo<'_>) {
        self.iterations
            .push((info.depth, info.score, info.pv.to_vec()));
    }

    fn best_move(&mut self, mv: Move) {
        self.best_moves.push(mv);
    }
}

/// The caller keeps a clone to read the reporter during or after a search
/// that owns the other one. A poisoned lock is still used: reporting never
/// panics in the search thread.
impl<R: Reporter> Reporter for Arc<Mutex<R>> {
    fn iteration(&mut self, info: &IterationInfo<'_>) {
        self.lock()
            .unwrap_or_else(PoisonError::into_inner)
            .iteration(info);
    }

    fn best_move(&mut self, mv: Move) {
        self.lock()
            .unwrap_or_else(PoisonError::into_inner)
            .best_move(mv);
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use engine::Square;

    use super::*;

    const E4: Move = Move::double_push(Square::E2, Square::E4);
    const D4: Move = Move::double_push(Square::D2, Square::D4);
    const E5: Move = Move::double_push(Square::E7, Square::E5);

    fn info(depth: u32, score: i32, pv: &[Move]) -> IterationInfo<'_> {
        IterationInfo {
            depth,
            seldepth: depth,
            score: Score::new(score),
            nodes: 0,
            time_ms: 0,
            pv,
        }
    }

    /// The reports of a short search: three iterations, then the best move.
    fn report(r: &mut dyn Reporter) {
        r.iteration(&info(1, 30, &[E4]));
        r.iteration(&info(2, -5, &[D4, E5]));
        r.iteration(&info(3, 12, &[E4, E5, D4]));
        r.best_move(E4);
    }

    fn recorded() -> RecordingReporter {
        RecordingReporter {
            iterations: vec![
                (1, Score::new(30), vec![E4]),
                (2, Score::new(-5), vec![D4, E5]),
                (3, Score::new(12), vec![E4, E5, D4]),
            ],
            best_moves: vec![E4],
        }
    }

    #[test]
    fn recording_keeps_reports_in_order() {
        let mut r = RecordingReporter::default();
        report(&mut r);
        assert_eq!(r, recorded());
    }

    #[test]
    fn recording_keeps_every_best_move() {
        let mut r = RecordingReporter::default();
        r.best_move(Move::NULL);
        r.best_move(E4);
        assert_eq!(r.best_moves, [Move::NULL, E4]);
        assert!(r.iterations.is_empty());
    }

    #[test]
    fn silent_accepts_reports() {
        report(&mut SilentReporter);
    }

    #[test]
    fn shared_reporter_is_read_from_another_thread() {
        let shared = Arc::new(Mutex::new(RecordingReporter::default()));
        let mut boxed: Box<dyn Reporter> = Box::new(Arc::clone(&shared));
        thread::spawn(move || report(boxed.as_mut()))
            .join()
            .unwrap();
        assert_eq!(*shared.lock().unwrap(), recorded());
    }

    #[test]
    fn poisoned_reporter_still_records() {
        let mut shared = Arc::new(Mutex::new(RecordingReporter::default()));
        let poisoner = Arc::clone(&shared);
        let result = thread::spawn(move || {
            let _guard = poisoner.lock().unwrap();
            panic!("poison the lock");
        })
        .join();
        assert!(result.is_err() && shared.is_poisoned());
        report(&mut shared);
        let r = shared.lock().unwrap_or_else(PoisonError::into_inner);
        assert_eq!(*r, recorded());
    }
}
