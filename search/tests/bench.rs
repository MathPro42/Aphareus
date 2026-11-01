use search::bench::{self, BENCH_DEPTH};

/// Runs the bench twice and checks both runs searched the same nodes.
fn assert_deterministic(depth: u32) {
    let first = bench::run(depth);
    let second = bench::run(depth);
    assert_eq!(first.nodes_per_position.len(), 50);
    assert_eq!(first.nodes_per_position, second.nodes_per_position);
    assert_eq!(first.nodes, second.nodes);
    assert_eq!(first.nodes, first.nodes_per_position.iter().sum::<u64>());
}

#[test]
fn bench_is_deterministic() {
    assert_deterministic(2);
}

#[test]
#[ignore = "slow: run in release"]
fn full_bench_is_deterministic() {
    assert_deterministic(BENCH_DEPTH);
}
