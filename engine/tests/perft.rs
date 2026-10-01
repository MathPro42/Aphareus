mod common;

use std::thread;

use common::{CHESS960, PerftCase, STANDARD, load};
use engine::{Board, divide, perft};

const STANDARD_BUDGET: u64 = 1_000_000;
const CHESS960_BUDGET: u64 = 100_000;

fn check(cases: &[PerftCase], budget: u64) {
    for case in cases {
        for &(depth, nodes) in case.counts.iter().filter(|&&(_, n)| n <= budget) {
            assert_eq!(
                perft(&case.board, depth),
                nodes,
                "{} depth {depth}",
                case.fen
            );
        }
    }
}

#[test]
fn standard_suite() {
    check(&load(STANDARD), STANDARD_BUDGET);
}

#[test]
fn chess960_suite() {
    check(&load(CHESS960), CHESS960_BUDGET);
}

#[test]
fn perft_zero_is_one() {
    assert_eq!(perft(&Board::startpos(), 0), 1);
}

#[test]
fn divide_sums_to_perft() {
    let board = Board::startpos();
    let split = divide(&board, 3);
    assert_eq!(split.len(), 20);
    assert_eq!(split.iter().map(|&(_, n)| n).sum::<u64>(), 8_902);
    for (m, nodes) in split {
        assert_eq!(nodes, perft(&board.make_move(m), 2), "{m:?}");
    }
}

#[test]
#[ignore = "long: run in release with --ignored"]
fn full_suites() {
    let mut cases = load(STANDARD);
    cases.extend(load(CHESS960));
    let threads = thread::available_parallelism().map_or(4, |n| n.get());
    let failures: Vec<String> = thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|t| {
                let cases = &cases;
                scope.spawn(move || {
                    let mut failures = Vec::new();
                    for case in cases.iter().skip(t).step_by(threads) {
                        for &(depth, nodes) in &case.counts {
                            let got = perft(&case.board, depth);
                            if got != nodes {
                                failures.push(format!(
                                    "{} depth {depth}: {got} instead of {nodes}",
                                    case.fen
                                ));
                            }
                        }
                    }
                    failures
                })
            })
            .collect();
        workers
            .into_iter()
            .flat_map(|w| w.join().unwrap())
            .collect()
    });
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
