use std::time::{Duration, Instant};

use engine::Board;

/// `perft <depth>`: the leaf count, its time and speed, as an
/// `info string` so that a strict GUI ignores it.
pub fn perft(board: &Board, depth: u32) -> String {
    let start = Instant::now();
    let nodes = engine::perft(board, depth);
    let elapsed = start.elapsed();
    format!(
        "info string perft({depth}) = {nodes} ({} ms, {} nps)",
        elapsed.as_millis(),
        nps(nodes, elapsed)
    )
}

/// `d`: the grid, FEN, hash and checkers of `board`.
pub fn display(board: &Board) -> String {
    board.to_string().trim_end().to_string()
}

/// Nodes per second. Counted in nanoseconds: small perfts take under 1 ms.
pub fn nps(nodes: u64, elapsed: Duration) -> u64 {
    let nps = u128::from(nodes) * 1_000_000_000 / elapsed.as_nanos().max(1);
    u64::try_from(nps).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::from_uci;

    #[test]
    fn perft_gives_the_known_count() {
        let line = perft(&Board::startpos(), 3);
        assert!(line.starts_with("info string perft(3) = 8902 ("), "{line}");
        assert!(line.ends_with(" nps)"), "{line}");
    }

    #[test]
    fn display_shows_the_fen() {
        let start = Board::startpos();
        let board = start.make_move(from_uci(&start, "e2e4", false).unwrap());
        let text = display(&board);
        assert!(text.contains(&board.to_fen()), "{text}");
        assert!(!text.ends_with('\n'));
    }

    #[test]
    fn nps_values() {
        assert_eq!(nps(1_000, Duration::from_millis(1)), 1_000_000);
        assert_eq!(nps(20, Duration::from_nanos(100)), 200_000_000);
        assert_eq!(nps(0, Duration::from_secs(1)), 0);
    }

    #[test]
    fn nps_without_time_does_not_divide_by_zero() {
        assert_eq!(nps(5, Duration::ZERO), 5_000_000_000);
        assert_eq!(nps(u64::MAX, Duration::ZERO), u64::MAX);
    }
}
