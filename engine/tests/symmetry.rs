mod common;

use common::{CHESS960, PerftCase, STANDARD, load};
use engine::perft;

const STANDARD_BUDGET: u64 = 1_000_000;
const CHESS960_BUDGET: u64 = 100_000;

fn check(cases: &[PerftCase], budget: u64) {
    for case in cases {
        let mirrored = case.board.mirror();
        for &(depth, nodes) in case.counts.iter().filter(|&&(_, n)| n <= budget) {
            assert_eq!(
                perft(&mirrored, depth),
                nodes,
                "mirror of {} depth {depth}",
                case.fen
            );
        }
    }
}

#[test]
fn standard_suite_mirrored() {
    check(&load(STANDARD), STANDARD_BUDGET);
}

#[test]
fn chess960_suite_mirrored() {
    check(&load(CHESS960), CHESS960_BUDGET);
}
