use std::io::Write;
use std::sync::{Arc, Mutex, PoisonError};

use engine::{Board, Move, to_uci};
use eval::Score;
use search::{IterationInfo, Reporter};

/// The UCI output, shared by the main loop and the search thread.
pub type Output = Arc<Mutex<dyn Write + Send>>;

/// Writes `line` and flushes it: the GUI reads line by line.
pub fn send(output: &Output, line: &str) {
    // A writer poisoned by a panic still holds whole lines. A closed output
    // cannot be reported anywhere; the end of the input will follow.
    let mut output = output.lock().unwrap_or_else(PoisonError::into_inner);
    let _ = writeln!(output, "{line}");
    let _ = output.flush();
}

/// Writes the reports of one search as `info` and `bestmove` lines.
pub struct UciReporter {
    /// Moves are written from the board they are played on: castles depend
    /// on it.
    root: Board,
    chess960: bool,
    output: Output,
}

impl UciReporter {
    pub fn new(root: Board, chess960: bool, output: Output) -> UciReporter {
        UciReporter {
            root,
            chess960,
            output,
        }
    }
}

impl Reporter for UciReporter {
    fn iteration(&mut self, info: &IterationInfo<'_>) {
        send(&self.output, &info_line(&self.root, self.chess960, info));
    }

    fn best_move(&mut self, mv: Move) {
        send(
            &self.output,
            &format!("bestmove {}", move_text(&self.root, mv, self.chess960)),
        );
    }
}

/// `info depth … seldepth … score … nodes … nps … time … pv …`.
fn info_line(root: &Board, chess960: bool, info: &IterationInfo<'_>) -> String {
    let nps = info.nodes.saturating_mul(1000) / info.time_ms.max(1);
    format!(
        "info depth {} seldepth {} score {} nodes {} nps {nps} time {} pv {}",
        info.depth,
        info.seldepth,
        score_text(info.score),
        info.nodes,
        info.time_ms,
        pv_text(root, info.pv, chess960),
    )
}

/// `mate <moves>` (negative when mated) or `cp <centipawns>`.
fn score_text(score: Score) -> String {
    match score.mate_moves() {
        Some(moves) => format!("mate {moves}"),
        None => format!("cp {}", score.raw()),
    }
}

/// The moves of `pv`, each written from the board it is played on.
fn pv_text(root: &Board, pv: &[Move], chess960: bool) -> String {
    let mut board = root.clone();
    let mut moves = Vec::with_capacity(pv.len());
    for &mv in pv {
        moves.push(to_uci(&board, mv, chess960));
        board = board.make_move(mv);
    }
    moves.join(" ")
}

/// `0000` for `Move::NULL`: no legal move.
fn move_text(board: &Board, mv: Move, chess960: bool) -> String {
    if mv.is_null() {
        "0000".to_string()
    } else {
        to_uci(board, mv, chess960)
    }
}

#[cfg(test)]
mod tests {
    use engine::from_uci;

    use super::*;

    /// White can castle king side, in both notations.
    const CASTLE: &str = "4k3/8/8/8/8/8/8/4K2R w K - 0 1";

    /// The moves of `texts`, read one after the other from `root`.
    fn line(root: &Board, texts: &[&str], chess960: bool) -> Vec<Move> {
        let mut board = root.clone();
        texts
            .iter()
            .map(|text| {
                let mv = from_uci(&board, text, chess960).unwrap();
                board = board.make_move(mv);
                mv
            })
            .collect()
    }

    fn info(score: Score, nodes: u64, time_ms: u64, pv: &[Move]) -> IterationInfo<'_> {
        IterationInfo {
            depth: 3,
            seldepth: 7,
            score,
            nodes,
            time_ms,
            pv,
        }
    }

    #[test]
    fn scores() {
        assert_eq!(score_text(Score::new(35)), "cp 35");
        assert_eq!(score_text(Score::new(-120)), "cp -120");
        assert_eq!(score_text(Score::mate_in(1)), "mate 1");
        assert_eq!(score_text(Score::mate_in(5)), "mate 3");
        assert_eq!(score_text(Score::mated_in(2)), "mate -1");
    }

    #[test]
    fn info_line_fields() {
        let root = Board::startpos();
        let pv = line(&root, &["e2e4", "e7e5"], false);
        assert_eq!(
            info_line(&root, false, &info(Score::new(20), 5000, 10, &pv)),
            "info depth 3 seldepth 7 score cp 20 nodes 5000 nps 500000 time 10 pv e2e4 e7e5"
        );
    }

    #[test]
    fn nps_without_time_does_not_divide_by_zero() {
        let root = Board::startpos();
        let pv = line(&root, &["e2e4"], false);
        let text = info_line(&root, false, &info(Score::new(0), 42, 0, &pv));
        assert!(text.contains(" nodes 42 nps 42000 time 0 "), "{text}");
    }

    #[test]
    fn pv_moves_are_written_from_their_own_board() {
        let root = Board::from_fen(CASTLE).unwrap();
        // Black's reply only exists after the castle.
        let pv = line(&root, &["e1g1", "e8d7", "f1f7"], false);
        assert_eq!(pv_text(&root, &pv, false), "e1g1 e8d7 f1f7");
        let pv = line(&root, &["e1h1", "e8d7", "f1f7"], true);
        assert_eq!(pv_text(&root, &pv, true), "e1h1 e8d7 f1f7");
    }

    #[test]
    fn best_move_text() {
        let root = Board::from_fen(CASTLE).unwrap();
        let castle = from_uci(&root, "e1g1", false).unwrap();
        assert_eq!(move_text(&root, castle, false), "e1g1");
        assert_eq!(move_text(&root, castle, true), "e1h1");
        assert_eq!(move_text(&root, Move::NULL, false), "0000");
    }

    #[test]
    fn reporter_writes_info_then_bestmove() {
        let bytes = Arc::new(Mutex::new(Vec::new()));
        let output: Output = Arc::clone(&bytes) as Output;
        let root = Board::startpos();
        let pv = line(&root, &["d2d4"], false);
        let mut reporter = UciReporter::new(root, false, output);
        reporter.iteration(&info(Score::new(15), 20, 1, &pv));
        reporter.best_move(pv[0]);
        let text = String::from_utf8(bytes.lock().unwrap().clone()).unwrap();
        assert_eq!(
            text,
            "info depth 3 seldepth 7 score cp 15 nodes 20 nps 20000 time 1 pv d2d4\nbestmove d2d4\n"
        );
    }
}
