//! Temporary engine until the search is implemented plays a random legal move.

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use engine::movegen::legal_moves;
use engine::{Board, Color, Move, Prng};

use crate::uci::command::GoParams;

/// With a clock, a move takes this fraction of it.
const CLOCK_FRACTION: u32 = 20;

/// With a clock, a move never takes longer than this.
const MAX_CLOCK_TIME: Duration = Duration::from_secs(1);

/// How often the wait checks the stop flag.
const POLL: Duration = Duration::from_millis(5);

/// Picks a random legal move of `board`, then waits for the time `params`
/// allow or for `stop`, and returns it. Without a legal move, returns
/// `Move::NULL` at once.
pub fn play(board: &Board, params: &GoParams, stop: &AtomicBool) -> Move {
    let best = choose(board, &mut Prng::new(clock_seed()));
    if !best.is_null() {
        wait(think_time(params, board.side_to_move()), stop);
    }
    best
}

/// A random legal move, or `Move::NULL` without one.
fn choose(board: &Board, rng: &mut Prng) -> Move {
    let moves = legal_moves(board);
    let moves = moves.as_slice();
    if moves.is_empty() {
        return Move::NULL;
    }
    // At most 256 moves: the bias of `%` is negligible.
    moves[(rng.next_u64() % moves.len() as u64) as usize]
}

/// `infinite` waits for `stop`, `movetime` is followed, a clock gives a
/// small part of it. Anything else (`depth`, `nodes`, nothing) answers at
/// once.
fn think_time(params: &GoParams, us: Color) -> Duration {
    if params.infinite {
        Duration::MAX
    } else if let Some(time) = params.movetime {
        time
    } else if let Some(time) = params.time[us.index()] {
        (time / CLOCK_FRACTION).min(MAX_CLOCK_TIME)
    } else {
        Duration::ZERO
    }
}

/// Sleeps for `time`, or until `stop` is raised.
fn wait(time: Duration, stop: &AtomicBool) {
    let start = Instant::now();
    while !stop.load(Ordering::Relaxed) {
        let left = time.saturating_sub(start.elapsed());
        if left.is_zero() {
            return;
        }
        // Never past `time`: a short clock can be shorter than `POLL`.
        thread::sleep(left.min(POLL));
    }
}

/// A seed that changes from one game to the next, never zero.
fn clock_seed() -> u64 {
    let since_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    // Truncated on purpose: the low bits are the ones that change.
    since_epoch.as_nanos() as u64 | 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uci::command::{Command, parse};

    /// One legal move, `a1b2`: the white king is in check and must take.
    const ONE_MOVE: &str = "k7/8/8/8/8/8/1r6/K1r5 w - - 0 1";

    /// Black is checkmated.
    const MATED: &str = "k7/1Q6/1K6/8/8/8/8/8 b - - 0 1";

    /// The perft suites of `engine`: one FEN per line, then the counts.
    const SUITES: [&str; 2] = [
        include_str!("../../../engine/tests/data/perft_standard.epd"),
        include_str!("../../../engine/tests/data/perft_960.epd"),
    ];

    fn go(line: &str) -> GoParams {
        match parse(line) {
            Command::Go(params) => params,
            other => panic!("{line:?} gave {other:?}"),
        }
    }

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    #[test]
    fn a_single_legal_move_is_always_chosen() {
        let board = Board::from_fen(ONE_MOVE).unwrap();
        let moves = legal_moves(&board);
        assert_eq!(moves.len(), 1);
        for seed in 1..=1000 {
            assert_eq!(choose(&board, &mut Prng::new(seed)), moves.as_slice()[0]);
        }
    }

    #[test]
    fn seeds_cover_every_move() {
        let board = Board::startpos();
        let moves = legal_moves(&board);
        let mut seen = Vec::new();
        for seed in 1..=1000 {
            let m = choose(&board, &mut Prng::new(seed));
            if !seen.contains(&m) {
                seen.push(m);
            }
        }
        assert_eq!(seen.len(), moves.len());
    }

    #[test]
    fn chosen_moves_are_legal() {
        let mut rng = Prng::new(1);
        for suite in SUITES {
            for line in suite.lines().filter(|line| !line.trim().is_empty()) {
                let fen = line.split(';').next().unwrap().trim();
                let board = Board::from_fen(fen).unwrap();
                let children = legal_moves(&board)
                    .as_slice()
                    .iter()
                    .map(|&m| board.make_move(m))
                    .collect::<Vec<_>>();
                for board in [&board].into_iter().chain(&children) {
                    let moves = legal_moves(board);
                    for _ in 0..100 {
                        let m = choose(board, &mut rng);
                        // Some children are mates: `NULL` without a legal move.
                        let legal = if moves.is_empty() {
                            m.is_null()
                        } else {
                            moves.contains(m)
                        };
                        assert!(legal, "{m:?} in {}", board.to_fen());
                    }
                }
            }
        }
    }

    #[test]
    fn no_legal_move_gives_null_at_once() {
        let board = Board::from_fen(MATED).unwrap();
        let start = Instant::now();
        let best = play(&board, &go("go movetime 5000"), &AtomicBool::new(false));
        assert!(best.is_null());
        assert!(start.elapsed() < ms(100), "took {:?}", start.elapsed());
    }

    #[test]
    fn think_time_follows_the_limits() {
        let white = Color::White;
        assert_eq!(think_time(&go("go infinite"), white), Duration::MAX);
        assert_eq!(think_time(&go("go movetime 100"), white), ms(100));
        assert_eq!(think_time(&go("go wtime 60000"), white), MAX_CLOCK_TIME);
        assert_eq!(think_time(&go("go wtime 2000"), white), ms(100));
        assert_eq!(think_time(&go("go depth 5"), white), Duration::ZERO);
        assert_eq!(think_time(&go("go nodes 1000"), white), Duration::ZERO);
        assert_eq!(think_time(&go("go"), white), Duration::ZERO);
    }

    #[test]
    fn think_time_reads_the_clock_of_the_side_to_move() {
        let params = go("go wtime 60000 btime 2000");
        assert_eq!(think_time(&params, Color::White), MAX_CLOCK_TIME);
        assert_eq!(think_time(&params, Color::Black), ms(100));
    }

    #[test]
    fn wait_lasts_the_given_time() {
        let start = Instant::now();
        wait(ms(20), &AtomicBool::new(false));
        assert!(start.elapsed() >= ms(20));
    }

    #[test]
    fn stop_ends_the_wait() {
        let board = Board::startpos();
        let stop = AtomicBool::new(false);
        let start = Instant::now();
        let best = thread::scope(|s| {
            let search = s.spawn(|| play(&board, &go("go movetime 5000"), &stop));
            stop.store(true, Ordering::Relaxed);
            search.join().unwrap()
        });
        assert!(start.elapsed() < ms(100), "took {:?}", start.elapsed());
        assert!(legal_moves(&board).contains(best));
    }
}
