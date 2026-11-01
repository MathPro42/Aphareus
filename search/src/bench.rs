//! Fixed-depth searches of fixed positions. The total node count identifies
//! the exact behavior of the search.

use std::panic;
use std::thread;
use std::time::{Duration, Instant};

use engine::{Board, Position};

use crate::Limits;
use crate::report::SilentReporter;
use crate::root;
use crate::shared::Shared;
use crate::thread::{STACK_SIZE, ThreadState};

/// The depth of `aphareus bench`. Changing it changes the bench count.
pub const BENCH_DEPTH: u32 = 5;

/// Perft positions, Chess960, openings, middlegames, endgames and short
/// mates. Changing them changes the bench count.
const BENCH_FENS: [&str; 50] = [
    // Perft suite
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
    "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
    "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1",
    "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
    "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
    // Chess960
    "bqnb1rkr/pp3ppp/3ppn2/2p5/5P2/P2P4/NPP1P1PP/BQ1BNRKR w HFhf - 2 9",
    "nb1r1nkr/ppp1ppp1/2bp4/7p/3P2qP/P6R/1PP1PPP1/NBQRBNK1 w Dhd - 1 9",
    "1bnrkqnr/p1pppp2/7p/1p4p1/4b3/7N/PPPP1PPP/BBNRKQ1R w HDhd - 0 9",
    "nbrk1rqn/p1ppp2p/1p6/5ppb/8/1N2P2P/PPPP1PP1/1BKRBRQN w fc - 0 9",
    "bbqr1knr/pppppp1p/8/4n1p1/2P1P3/6P1/PPQP1P1P/BB1RNKNR w HDhd - 0 9",
    "rbqnb1rn/p1pp1kpp/1p2pp2/8/4P2P/P5P1/1PPP1P2/RBQNBKRN w GA - 0 9",
    "bbrk1q1r/ppppppp1/3n4/7p/3Pn3/6PN/PPP1PPNP/BBRK1Q1R w HChc - 2 9",
    "rbk1brqn/ppp1pppp/8/3p4/7P/1P4P1/2PPPP2/RBKNBRQN w FAfa - 0 9",
    // Openings
    "r1b2rk1/2q1bppp/p2p1n2/npp1p3/3PP3/2P2N1P/PPB2PP1/RNBQR1K1 w - - 1 12",
    "r2q1rk1/3nbppp/p2pbn2/1p2p3/4P1P1/1NN1BP2/PPPQ3P/2KR1B1R w - - 0 12",
    "r1bq1rk1/pp1nbppp/2p1p3/3n2B1/2BP4/2N1PN2/PP3PPP/2RQK2R w K - 1 10",
    "r1bq1rk1/pppnn1bp/3p2p1/3Ppp2/2P1P3/2NN4/PP2BPPP/R1BQ1RK1 w - - 0 11",
    "rnb1k1r1/ppq1np1Q/4p3/3pP3/3p4/P1P5/2P2PPP/R1B1KBNR w KQq - 0 10",
    "r2qkbnr/pp1n1pp1/2p1p2p/7P/3P4/3Q1NN1/PPP2PP1/R1B1K2R w KQkq - 0 11",
    "r1bq1rk1/pp2npbp/2n1p1p1/2pp4/2P5/2NP1NP1/PP1BPPBP/R2Q1RK1 w - - 0 9",
    "r1bq1rk1/bpp2pp1/p1np1n1p/4p3/P1B1P3/2PP1N1P/1P3PP1/RNBQR1K1 w - - 0 10",
    "r1bq1rk1/pp3ppp/2n1pn2/b1p5/2BP4/P1N1PN2/1P3PPP/R1BQ1RK1 w - - 1 10",
    "r1bq1rk1/pp2ppbp/2n3p1/8/2BPP3/4B3/P3NPPP/R2Q1RK1 b - - 0 11",
    "2rq1rk1/pp1bppb1/3p1np1/4n2p/3NP2P/1BN1BP2/PPPQ2P1/2KR3R w - - 0 13",
    "r2q1rk1/pp1n1ppp/2p1pn2/5b2/PbBP4/2N1PN2/1P2QPPP/R1B2RK1 w - - 5 10",
    "2kr1b1r/p1ppqppp/b1p5/3nP3/2P5/1P6/P3QPPP/RNB1KB1R w KQ - 1 10",
    "rnbqr1k1/pp3pbp/3p1np1/2pP4/4P3/2N2N2/PP2BPPP/R1BQ1RK1 w - - 6 10",
    "r1bq1rk1/p4ppp/1pnbpn2/2pp4/3P4/2PBPNB1/PP1N1PPP/R2QK2R w KQ - 0 9",
    "rnbqk2r/ppp2p1p/3b1n2/3PN3/2B2ppP/8/PPPP2P1/RNBQK2R w KQkq - 1 8",
    // Middlegames
    "r1bq1rk1/pp2bppp/2n1pn2/3p4/2PP4/2N1PN2/PP3PPP/R2QKB1R w KQ - 0 9",
    "2r2rk1/pp1bqppp/2n1pn2/3p4/3P4/2PBPN2/P1Q2PPP/R1B2RK1 w - - 0 12",
    "r2q1rk1/pb1nbppp/1p2pn2/2pp4/3P4/1P1BPN2/PBPN1PPP/R2Q1RK1 b - - 0 10",
    "r2q1rk1/pp3ppp/8/8/2QQ4/2QQ1N2/PP2BPPP/RNB2RK1 w - - 0 15",
    // Endgames
    "8/pp3k2/8/3P4/8/8/PP3K2/8 b - - 0 40",
    "8/8/8/8/3pP3/8/8/4K2k b - e3 0 1",
    "8/5pk1/6p1/8/3R4/6P1/r4PK1/8 w - - 0 40",
    "8/8/3k4/8/3K4/2B5/8/6n1 b - - 0 50",
    "1K6/1P1k4/8/8/8/8/r7/2R5 w - - 0 1",
    "4k3/8/8/4PK2/8/r7/8/7R b - - 0 1",
    "8/1P6/8/8/8/8/6p1/K6k w - - 0 1",
    "8/8/8/3k4/8/8/2r5/K3Q3 w - - 0 1",
    "8/8/8/4k3/8/8/8/4KBN1 w - - 0 1",
    "8/4kp2/6p1/3b4/5P2/4B1P1/5K2/8 w - - 0 1",
    "8/5pk1/6p1/2n5/8/5PP1/4N1K1/8 b - - 0 1",
    "3q2k1/5pp1/7p/8/8/6PP/5PK1/3Q4 w - - 0 1",
    // Short mates
    "r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 4 4",
    "rn1qkbnr/ppp2p1p/3p2p1/4N3/2B1P3/2N5/PPPP1PPP/R1BbK2R w KQkq - 0 6",
    "4kb1r/p2n1ppp/4q3/4p1B1/4P3/1Q6/PPP2PPP/2KR4 w k - 0 16",
];

/// The result of one bench.
#[derive(Clone, Debug)]
pub struct BenchResult {
    /// Nodes searched in each position, in order.
    pub nodes_per_position: Vec<u64>,
    /// Total node count: the bench number.
    pub nodes: u64,
    pub time: Duration,
}

/// Searches every bench position to `depth`, one after the other, with one
/// thread and no time limit. The node count is the same on every run and
/// every platform.
///
/// # Panics
///
/// If the system cannot create the bench thread.
#[must_use]
pub fn run(depth: u32) -> BenchResult {
    // The search is recursive: it gets the same large stack as in the pool.
    thread::scope(|s| {
        thread::Builder::new()
            .name("bench".to_owned())
            .stack_size(STACK_SIZE)
            .spawn_scoped(s, || run_fens(&BENCH_FENS, depth))
            .expect("cannot create the bench thread")
            .join()
            .unwrap_or_else(|e| panic::resume_unwind(e))
    })
}

/// Searches each of `fens` to `depth`. Nothing carries over from one
/// position to the next.
fn run_fens(fens: &[&str], depth: u32) -> BenchResult {
    let mut t = Box::new(ThreadState::new());
    let shared = Shared::new();
    let limits = Limits::depth(depth);
    let start = Instant::now();
    let nodes_per_position: Vec<u64> = fens
        .iter()
        .map(|fen| {
            let board = Board::from_fen(fen).expect("bench positions are valid");
            t.new_game();
            root::search(
                &mut t,
                &shared,
                Position::new(board),
                &limits,
                &mut SilentReporter,
            );
            t.nodes
        })
        .collect();
    BenchResult {
        nodes: nodes_per_position.iter().sum(),
        nodes_per_position,
        time: start.elapsed(),
    }
}

#[cfg(test)]
mod tests {
    use engine::movegen::legal_moves;

    use super::*;

    #[test]
    fn positions_are_valid_and_not_terminal() {
        for fen in BENCH_FENS {
            let board = Board::from_fen(fen).unwrap_or_else(|e| panic!("{fen}: {e}"));
            assert!(!legal_moves(&board).is_empty(), "{fen}");
        }
    }

    #[test]
    fn positions_are_distinct() {
        for (i, a) in BENCH_FENS.iter().enumerate() {
            assert!(!BENCH_FENS[i + 1..].contains(a), "{a}");
        }
    }

    #[test]
    fn nothing_carries_over_between_positions() {
        let forward = run_fens(&BENCH_FENS, 2);
        let mut reversed = BENCH_FENS;
        reversed.reverse();
        let mut backward = run_fens(&reversed, 2).nodes_per_position;
        backward.reverse();
        assert_eq!(forward.nodes_per_position, backward);
    }

    #[test]
    fn total_is_the_sum_of_the_positions() {
        let result = run_fens(&BENCH_FENS[..3], 1);
        assert_eq!(result.nodes_per_position.len(), 3);
        assert!(result.nodes_per_position.iter().all(|&n| n > 0));
        assert_eq!(result.nodes, result.nodes_per_position.iter().sum());
    }
}
