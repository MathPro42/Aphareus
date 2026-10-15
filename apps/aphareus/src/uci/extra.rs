use std::time::{Duration, Instant};

use engine::{Board, Color};
use eval::{Evaluator, Taper, trace};

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

/// `eval`: the evaluation term by term, then the score from White's and from
/// the side to move's point of view. Raw lines for a human, like `d`.
pub fn eval(board: &Board) -> String {
    let t = trace(board);
    let (white, black) = (Color::White, Color::Black);
    let rule = "--------------+-----------------+-----------------+-----------------";
    let mg_eg = format!(" {:>7} {:>7} ", "MG", "EG");
    let lines = [
        format!(
            " {:<12} |{:^17}|{:^17}|{:^17}",
            "Term", "White", "Black", "Total"
        ),
        format!(" {:<12} |{mg_eg}|{mg_eg}|{mg_eg}", ""),
        rule.to_string(),
        row("Material", t.material(white), t.material(black)),
        row("Piece-square", t.piece_square(white), t.piece_square(black)),
        rule.to_string(),
        row(
            "Total",
            t.material(white) + t.piece_square(white),
            t.material(black) + t.piece_square(black),
        ),
        String::new(),
        format!("Phase: {} / 24", t.phase()),
        format!("Score (White): {} cp", t.total()),
        format!(
            "Score (side to move): {} cp",
            Evaluator::new().evaluate(board).raw()
        ),
    ];
    lines.join("\n")
}

/// One table row: White, Black, then White minus Black.
fn row(label: &str, white: Taper, black: Taper) -> String {
    format!(
        " {label:<12} |{}|{}|{}",
        cells(white),
        cells(black),
        cells(white - black)
    )
}

/// The middlegame and endgame values of `t`, 17 characters wide.
fn cells(t: Taper) -> String {
    format!(" {:>7} {:>7} ", t.mg(), t.eg())
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

    const KIWIPETE: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";

    /// The number at the end of the line that starts with `prefix`.
    fn score(text: &str, prefix: &str) -> i32 {
        let line = text
            .lines()
            .find(|line| line.starts_with(prefix))
            .unwrap_or_else(|| panic!("no {prefix:?} line in {text}"));
        line[prefix.len()..]
            .trim_end_matches(" cp")
            .parse()
            .unwrap()
    }

    /// The six numbers of the table row named `label`.
    fn row_values(text: &str, label: &str) -> Vec<i32> {
        let line = text
            .lines()
            .find(|line| line.split('|').next().map(str::trim) == Some(label))
            .unwrap_or_else(|| panic!("no {label:?} row in {text}"));
        line.split('|')
            .skip(1)
            .flat_map(str::split_whitespace)
            .map(|v| v.parse().unwrap())
            .collect()
    }

    #[test]
    fn eval_startpos_is_zero() {
        let text = eval(&Board::startpos());
        assert!(text.lines().any(|line| line == "Phase: 24 / 24"), "{text}");
        assert_eq!(score(&text, "Score (White): "), 0);
        assert_eq!(score(&text, "Score (side to move): "), 0);
        assert!(!text.ends_with('\n'));
    }

    #[test]
    fn eval_side_to_move_is_the_opposite_for_black() {
        let start = Board::startpos();
        let board = start.make_move(from_uci(&start, "e2e4", false).unwrap());
        let text = eval(&board);
        let white = score(&text, "Score (White): ");
        assert_ne!(white, 0, "{text}");
        assert_eq!(score(&text, "Score (side to move): "), -white, "{text}");
    }

    #[test]
    fn eval_rows_show_the_trace() {
        let board = Board::from_fen(KIWIPETE).unwrap();
        let t = trace(&board);
        let values = |w: Taper, b: Taper| {
            let d = w - b;
            vec![w.mg(), w.eg(), b.mg(), b.eg(), d.mg(), d.eg()]
        };
        let (white, black) = (Color::White, Color::Black);
        let text = eval(&board);
        assert_eq!(
            row_values(&text, "Material"),
            values(t.material(white), t.material(black))
        );
        assert_eq!(
            row_values(&text, "Piece-square"),
            values(t.piece_square(white), t.piece_square(black))
        );
        assert_eq!(
            row_values(&text, "Total"),
            values(
                t.material(white) + t.piece_square(white),
                t.material(black) + t.piece_square(black)
            )
        );
        assert_eq!(score(&text, "Score (White): "), t.total());
    }

    #[test]
    fn eval_table_columns_line_up() {
        let text = eval(&Board::from_fen(KIWIPETE).unwrap());
        let table: Vec<&str> = text.lines().take_while(|line| !line.is_empty()).collect();
        assert_eq!(table.len(), 7, "{text}");
        let separators =
            |line: &str| -> Vec<usize> { line.match_indices(['|', '+']).map(|(i, _)| i).collect() };
        for line in &table {
            assert_eq!(line.len(), table[0].len(), "{text}");
            assert_eq!(separators(line), separators(table[0]), "{text}");
        }
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
