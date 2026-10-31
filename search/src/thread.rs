use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};

use engine::{Board, Color, Position};
use eval::Evaluator;

use crate::Limits;
use crate::pv::PvTable;
use crate::report::Reporter;
use crate::root;
use crate::shared::Shared;
use crate::stack::Stack;
use crate::time::TimeManager;

/// The search is recursive: its thread gets a large stack. Only the pages
/// actually touched use memory.
const STACK_SIZE: usize = 64 << 20;

/// The stop flag and the hard time limit are read once every this many nodes.
#[cfg_attr(not(test), expect(dead_code, reason = "used by the search"))]
const CHECK_INTERVAL: u64 = 1024;

/// Everything one search thread owns, without lock or atomic. Allocated once
/// per thread (more than 128 KiB) and reused by every search.
pub(crate) struct ThreadState {
    /// Boards and hashes from the root of the game.
    pub(crate) position: Position,
    #[expect(dead_code, reason = "read by the search")]
    pub(crate) evaluator: Evaluator,
    pub(crate) stack: Stack,
    #[expect(dead_code, reason = "read by the search")]
    pub(crate) pv: PvTable,
    /// Local node count.
    pub(crate) nodes: u64,
    /// Highest ply reached, quiescence included.
    pub(crate) seldepth: u32,
    pub(crate) time: TimeManager,
    pub(crate) node_limit: Option<u64>,
    /// Set as soon as the search must stop.
    pub(crate) stopped: bool,
    /// False during iteration 1, which is never interrupted.
    pub(crate) can_stop: bool,
}

impl ThreadState {
    pub(crate) fn new() -> ThreadState {
        ThreadState {
            position: Position::new(Board::startpos()),
            evaluator: Evaluator::new(),
            stack: Stack::new(),
            pv: PvTable::new(),
            nodes: 0,
            seldepth: 0,
            time: TimeManager::new(&Limits::default(), Color::White),
            node_limit: None,
            stopped: false,
            can_stop: false,
        }
    }

    /// Forgets what survives from one search to the next: `ucinewgame`, and
    /// each bench position.
    pub(crate) fn new_game(&mut self) {
        self.stack.clear();
    }

    /// Sets `stopped` if the search must stop. Called at every node, costly
    /// only once every `CHECK_INTERVAL` nodes. The node limit is tested at
    /// every node, so a search limited in nodes is exactly reproducible.
    #[cfg_attr(not(test), expect(dead_code, reason = "used by the search"))]
    #[inline]
    pub(crate) fn check_stop(&mut self, shared: &Shared) {
        if !self.can_stop {
            return;
        }
        if self.node_limit.is_some_and(|limit| self.nodes >= limit) {
            self.stopped = true;
            return;
        }
        if self.nodes.is_multiple_of(CHECK_INTERVAL)
            && (shared.stop_requested() || self.time.hard_exceeded())
        {
            self.stopped = true;
        }
    }
}

#[expect(
    clippy::large_enum_variant,
    reason = "one task per go: its size does not matter"
)]
enum Task {
    Search {
        position: Position,
        limits: Limits,
        reporter: Box<dyn Reporter>,
    },
    NewGame,
    Quit,
}

/// The search threads: a single one for now. Created once, reused by
/// every `go`. Never more than one search at a time.
pub struct SearchPool {
    shared: Arc<Shared>,
    tasks: Sender<Task>,
    /// One message at the end of each search.
    done: Receiver<()>,
    busy: bool,
    /// `None` once the thread has been joined.
    handle: Option<JoinHandle<()>>,
}

impl SearchPool {
    /// Starts the search thread.
    ///
    /// # Panics
    ///
    /// If the system cannot create the thread.
    #[must_use]
    pub fn new() -> SearchPool {
        let shared = Arc::new(Shared::new());
        let (tasks, task_rx) = mpsc::channel();
        let (done_tx, done) = mpsc::channel();
        let worker_shared = Arc::clone(&shared);
        let handle = thread::Builder::new()
            .name("search".to_owned())
            .stack_size(STACK_SIZE)
            .spawn(move || work(&worker_shared, &task_rx, &done_tx))
            .expect("cannot create the search thread");
        SearchPool {
            shared,
            tasks,
            done,
            busy: false,
            handle: Some(handle),
        }
    }

    /// Starts a search and returns at once. A running search is stopped and
    /// waited for first.
    pub fn start(&mut self, position: Position, limits: Limits, reporter: Box<dyn Reporter>) {
        self.stop();
        self.wait();
        self.shared.clear_stop();
        let task = Task::Search {
            position,
            limits,
            reporter,
        };
        // An error means the thread is gone: there is nothing to wait for.
        self.busy = self.tasks.send(task).is_ok();
    }

    /// Asks the search to stop and returns at once. The search still
    /// reports its best move.
    pub fn stop(&self) {
        self.shared.request_stop();
    }

    /// Waits for the running search, if any, to end.
    pub fn wait(&mut self) {
        if self.busy {
            // An error means the thread is gone: it will not answer.
            let _ = self.done.recv();
            self.busy = false;
        }
    }

    /// Forgets what previous searches kept. A running search is stopped and
    /// waited for first.
    pub fn new_game(&mut self) {
        self.stop();
        self.wait();
        let _ = self.tasks.send(Task::NewGame);
    }

    /// Stops the search and ends the thread.
    pub fn quit(mut self) {
        self.shutdown();
    }

    /// Safe to call twice: `quit`, then `drop`.
    fn shutdown(&mut self) {
        let Some(handle) = self.handle.take() else {
            return;
        };
        self.stop();
        self.wait();
        let _ = self.tasks.send(Task::Quit);
        let _ = handle.join();
    }
}

impl Default for SearchPool {
    fn default() -> SearchPool {
        SearchPool::new()
    }
}

/// Dropping the pool, even during a search, ends the thread cleanly.
impl Drop for SearchPool {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// The loop of the search thread. Ends on `Quit`, or when the pool is gone.
fn work(shared: &Shared, tasks: &Receiver<Task>, done: &Sender<()>) {
    let mut t = Box::new(ThreadState::new());
    for task in tasks {
        match task {
            Task::Search {
                position,
                limits,
                mut reporter,
            } => {
                root::search(&mut t, shared, position, &limits, reporter.as_mut());
                // Released before `done`: after `wait`, the caller holds its
                // reporter alone.
                drop(reporter);
                if done.send(()).is_err() {
                    return;
                }
            }
            Task::NewGame => t.new_game(),
            Task::Quit => return,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::thread::ThreadId;
    use std::time::{Duration, Instant};

    use engine::movegen::legal_moves;
    use engine::{Move, Square};
    use eval::Score;

    use super::*;
    use crate::{IterationInfo, RecordingReporter};

    /// A pool call that takes longer than this has hung.
    const TIMEOUT: Duration = Duration::from_secs(5);

    fn startpos() -> Position {
        Position::new(Board::startpos())
    }

    fn infinite() -> Limits {
        Limits {
            infinite: true,
            ..Limits::default()
        }
    }

    fn recorder() -> (Arc<Mutex<RecordingReporter>>, Box<dyn Reporter>) {
        let rec = Arc::new(Mutex::new(RecordingReporter::default()));
        let reporter = Box::new(Arc::clone(&rec));
        (rec, reporter)
    }

    fn best_moves(rec: &Mutex<RecordingReporter>) -> Vec<Move> {
        rec.lock().unwrap().best_moves.clone()
    }

    /// Runs `f` in another thread and fails if it hangs.
    fn within_timeout<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || tx.send(f()));
        rx.recv_timeout(TIMEOUT).expect("hung or panicked")
    }

    /// Records the thread each best move comes from.
    #[derive(Default)]
    struct ThreadRecorder(Vec<ThreadId>);

    impl Reporter for ThreadRecorder {
        fn iteration(&mut self, _info: &IterationInfo<'_>) {}

        fn best_move(&mut self, _mv: Move) {
            self.0.push(thread::current().id());
        }
    }

    #[test]
    fn one_thread_serves_every_search() {
        let mut pool = SearchPool::new();
        let ids = Arc::new(Mutex::new(ThreadRecorder::default()));
        for i in 1..=3 {
            pool.start(startpos(), Limits::depth(1), Box::new(Arc::clone(&ids)));
            pool.wait();
            assert_eq!(ids.lock().unwrap().0.len(), i);
        }
        let ids = &ids.lock().unwrap().0;
        assert!(ids.iter().all(|&id| id == ids[0]));
        assert_ne!(ids[0], thread::current().id());
    }

    #[test]
    fn best_move_is_legal() {
        let mut pool = SearchPool::new();
        let (rec, reporter) = recorder();
        pool.start(startpos(), Limits::depth(1), reporter);
        pool.wait();
        let best = best_moves(&rec);
        assert_eq!(best.len(), 1);
        assert!(legal_moves(&Board::startpos()).contains(best[0]));
    }

    #[test]
    fn no_legal_move_reports_null() {
        let mated = Board::from_fen("7k/6Q1/6K1/8/8/8/8/8 b - - 0 1").unwrap();
        let mut pool = SearchPool::new();
        let (rec, reporter) = recorder();
        pool.start(Position::new(mated), Limits::depth(1), reporter);
        pool.wait();
        assert_eq!(best_moves(&rec), [Move::NULL]);
    }

    #[test]
    fn stop_ends_an_infinite_search() {
        let mut pool = SearchPool::new();
        let (rec, reporter) = recorder();
        pool.start(startpos(), infinite(), reporter);
        thread::sleep(Duration::from_millis(50));
        assert!(best_moves(&rec).is_empty());

        let start = Instant::now();
        pool.stop();
        let pool = within_timeout(move || {
            pool.wait();
            pool
        });
        assert!(start.elapsed() < Duration::from_millis(500));
        assert_eq!(best_moves(&rec).len(), 1);
        drop(pool);
    }

    #[test]
    fn start_stops_the_running_search() {
        let mut pool = SearchPool::new();
        let (first, reporter) = recorder();
        pool.start(startpos(), infinite(), reporter);
        let (second, reporter) = recorder();
        let pool = within_timeout(move || {
            pool.start(startpos(), Limits::depth(1), reporter);
            pool.wait();
            pool
        });
        assert_eq!(best_moves(&first).len(), 1);
        assert_eq!(best_moves(&second).len(), 1);
        drop(pool);
    }

    #[test]
    fn new_game_stops_the_running_search() {
        let mut pool = SearchPool::new();
        let (first, reporter) = recorder();
        pool.start(startpos(), infinite(), reporter);
        let mut pool = within_timeout(move || {
            pool.new_game();
            pool
        });
        assert_eq!(best_moves(&first).len(), 1);

        let (second, reporter) = recorder();
        pool.start(startpos(), Limits::depth(1), reporter);
        pool.wait();
        assert_eq!(best_moves(&second).len(), 1);
    }

    #[test]
    fn quit_during_a_search_ends_the_thread() {
        let mut pool = SearchPool::new();
        let (rec, reporter) = recorder();
        pool.start(startpos(), infinite(), reporter);
        within_timeout(move || pool.quit());
        assert_eq!(best_moves(&rec).len(), 1);
    }

    #[test]
    fn dropping_a_busy_pool_does_not_hang() {
        let mut pool = SearchPool::new();
        let (rec, reporter) = recorder();
        pool.start(startpos(), infinite(), reporter);
        within_timeout(move || drop(pool));
        assert_eq!(best_moves(&rec).len(), 1);
    }

    /// Kills the search thread when it reports.
    struct PanickingReporter;

    impl Reporter for PanickingReporter {
        fn iteration(&mut self, _info: &IterationInfo<'_>) {}

        fn best_move(&mut self, _mv: Move) {
            panic!("the search thread dies");
        }
    }

    #[test]
    fn dead_search_thread_never_blocks_the_caller() {
        let mut pool = SearchPool::new();
        pool.start(startpos(), Limits::depth(1), Box::new(PanickingReporter));
        within_timeout(move || {
            pool.wait();
            let (rec, reporter) = recorder();
            pool.start(startpos(), Limits::depth(1), reporter);
            pool.wait();
            pool.new_game();
            pool.quit();
            assert!(best_moves(&rec).is_empty());
        });
    }

    #[test]
    fn new_game_clears_the_stack() {
        let mut t = Box::new(ThreadState::new());
        t.stack.at(0).mv = Move::quiet(Square::G1, Square::F3);
        t.stack.at(5).eval = Score::DRAW;
        t.new_game();
        assert_eq!(t.stack.at(0).mv, Move::NULL);
        assert_eq!(t.stack.at(5).eval, Score::NONE);
    }

    fn searching() -> Box<ThreadState> {
        let mut t = Box::new(ThreadState::new());
        t.can_stop = true;
        t
    }

    #[test]
    fn first_iteration_never_stops() {
        let shared = Shared::new();
        shared.request_stop();
        let mut t = Box::new(ThreadState::new());
        t.node_limit = Some(1);
        t.nodes = CHECK_INTERVAL;
        t.check_stop(&shared);
        assert!(!t.stopped);
    }

    #[test]
    fn no_limit_never_stops() {
        let shared = Shared::new();
        let mut t = searching();
        t.nodes = CHECK_INTERVAL;
        t.check_stop(&shared);
        assert!(!t.stopped);
    }

    #[test]
    fn node_limit_is_exact() {
        let shared = Shared::new();
        let mut t = searching();
        t.node_limit = Some(1000);
        t.nodes = 999;
        t.check_stop(&shared);
        assert!(!t.stopped);
        t.nodes = 1000;
        t.check_stop(&shared);
        assert!(t.stopped);
    }

    #[test]
    fn stop_flag_is_read_every_interval() {
        let shared = Shared::new();
        shared.request_stop();
        let mut t = searching();
        t.nodes = CHECK_INTERVAL + 1;
        t.check_stop(&shared);
        assert!(!t.stopped);
        t.nodes = 2 * CHECK_INTERVAL;
        t.check_stop(&shared);
        assert!(t.stopped);
    }

    #[test]
    fn hard_limit_is_read_every_interval() {
        let shared = Shared::new();
        let mut t = searching();
        let one_ms = Limits {
            movetime: Some(1),
            ..Limits::default()
        };
        t.time = TimeManager::new(&one_ms, Color::White);
        thread::sleep(Duration::from_millis(5));
        t.nodes = CHECK_INTERVAL + 1;
        t.check_stop(&shared);
        assert!(!t.stopped);
        t.nodes = CHECK_INTERVAL;
        t.check_stop(&shared);
        assert!(t.stopped);
    }
}
