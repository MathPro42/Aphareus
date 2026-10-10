//! UCI sessions end to end: `uci::run` reads from a pipe the test writes
//! to, so the input stays open while a search runs.

use std::io::{self, BufReader, PipeWriter, Write};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use aphareus::uci;
use engine::movegen::legal_moves;
use engine::{Board, from_uci};

/// Only reached when something is broken: a hang fails instead of blocking.
const TIMEOUT: Duration = Duration::from_secs(5);

/// How fast `bestmove` must follow `stop` or a short `go`.
const QUICK: Duration = Duration::from_millis(500);

/// One legal move, `a1b2`: the white king is in check and must take.
const ONE_MOVE: &str = "k7/8/8/8/8/8/1r6/K1r5 w - - 0 1";

/// Black is checkmated.
const MATED: &str = "k7/1Q6/1K6/8/8/8/8/8 b - - 0 1";

/// An output the test reads while `run` writes to it.
#[derive(Clone, Default)]
struct SharedOutput(Arc<Mutex<Vec<u8>>>);

impl Write for SharedOutput {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// `uci::run` in its own thread, with a pipe as input.
struct Engine {
    input: Option<PipeWriter>,
    output: SharedOutput,
    run: Option<JoinHandle<()>>,
}

impl Engine {
    fn start() -> Engine {
        let (reader, writer) = io::pipe().unwrap();
        let output = SharedOutput::default();
        let engine_output = output.clone();
        let run = thread::spawn(move || uci::run(BufReader::new(reader), engine_output));
        Engine {
            input: Some(writer),
            output,
            run: Some(run),
        }
    }

    fn send(&mut self, line: &str) {
        let input = self.input.as_mut().expect("the input is open");
        writeln!(input, "{line}").unwrap();
    }

    /// Closes the input: the engine reads its end.
    fn close(&mut self) {
        self.input = None;
    }

    fn lines(&self) -> Vec<String> {
        let bytes = self.output.0.lock().unwrap().clone();
        String::from_utf8(bytes)
            .unwrap()
            .lines()
            .map(str::to_string)
            .collect()
    }

    fn lines_starting_with(&self, prefix: &str) -> Vec<String> {
        let mut lines = self.lines();
        lines.retain(|line| line.starts_with(prefix));
        lines
    }

    /// Waits for `count` lines starting with `prefix`, and returns them.
    fn wait_for_count(&self, prefix: &str, count: usize) -> Vec<String> {
        let start = Instant::now();
        loop {
            let found = self.lines_starting_with(prefix);
            if found.len() >= count {
                return found;
            }
            assert!(
                start.elapsed() < TIMEOUT,
                "waiting for {count} {prefix:?} lines in {:?}",
                self.lines()
            );
            thread::sleep(Duration::from_millis(1));
        }
    }

    /// Waits for a line starting with `prefix`, and returns it.
    fn wait_for(&self, prefix: &str) -> String {
        self.wait_for_count(prefix, 1).remove(0)
    }

    /// Waits for `run` to return.
    fn wait_end(&mut self) {
        let run = self.run.take().expect("run is still running");
        let start = Instant::now();
        while !run.is_finished() {
            assert!(
                start.elapsed() < TIMEOUT,
                "run did not return; output: {:?}",
                self.lines()
            );
            thread::sleep(Duration::from_millis(1));
        }
        run.join().unwrap();
    }
}

/// The move of a `bestmove` line.
fn best_move(line: &str) -> &str {
    line.strip_prefix("bestmove ")
        .unwrap_or_else(|| panic!("not a bestmove line: {line:?}"))
}

/// Asserts that the `bestmove` line plays a legal move of `board`.
fn assert_legal(board: &Board, line: &str) {
    let text = best_move(line);
    assert!(
        from_uci(board, text, false).is_some(),
        "{text} is not legal in {}",
        board.to_fen()
    );
}

#[test]
fn uci_handshake() {
    let mut engine = Engine::start();
    engine.send("uci");
    engine.wait_for("uciok");
    let lines = engine.lines();
    assert!(lines[0].starts_with("id name Aphareus "), "{lines:?}");
    assert!(lines[1].starts_with("id author "), "{lines:?}");
    assert_eq!(lines.last().unwrap(), "uciok");
    engine.send("quit");
    engine.wait_end();
}

#[test]
fn isready_answers_readyok() {
    let mut engine = Engine::start();
    engine.send("isready");
    engine.wait_for("readyok");
    engine.send("quit");
    engine.wait_end();
}

#[test]
fn go_movetime_plays_a_legal_move_quickly() {
    let mut engine = Engine::start();
    engine.send("position startpos");
    let start = Instant::now();
    engine.send("go movetime 50");
    let line = engine.wait_for("bestmove");
    assert!(start.elapsed() < Duration::from_secs(1));
    assert_legal(&Board::startpos(), &line);
    engine.send("quit");
    engine.wait_end();
}

#[test]
fn go_depth_plays_from_the_position_after_the_moves() {
    let mut engine = Engine::start();
    engine.send("position startpos moves e2e4");
    engine.send("go depth 1");
    let line = engine.wait_for("bestmove");
    let start = Board::startpos();
    let after_e4 = start.make_move(from_uci(&start, "e2e4", false).unwrap());
    assert_legal(&after_e4, &line);
    engine.send("quit");
    engine.wait_end();
}

#[test]
fn go_infinite_searches_until_stop_and_answers_isready_meanwhile() {
    let mut engine = Engine::start();
    engine.send("position startpos");
    engine.send("go infinite");
    thread::sleep(Duration::from_millis(100));
    engine.send("isready");
    engine.wait_for("readyok");
    assert!(engine.lines_starting_with("bestmove").is_empty());

    let start = Instant::now();
    engine.send("stop");
    let line = engine.wait_for("bestmove");
    assert!(
        start.elapsed() < QUICK,
        "bestmove took {:?}",
        start.elapsed()
    );
    assert_legal(&Board::startpos(), &line);
    engine.send("quit");
    engine.wait_end();
    assert_eq!(engine.lines_starting_with("bestmove").len(), 1);
}

#[test]
fn quit_during_go_infinite_ends_the_search() {
    let mut engine = Engine::start();
    engine.send("go infinite");
    engine.send("quit");
    engine.wait_end();
    assert_eq!(engine.lines_starting_with("bestmove").len(), 1);
}

#[test]
fn go_or_ucinewgame_during_go_infinite_ends_the_search() {
    let mut engine = Engine::start();
    engine.send("go infinite");
    engine.send("go movetime 10");
    engine.wait_for_count("bestmove", 2);
    engine.send("go infinite");
    engine.send("ucinewgame");
    engine.wait_for_count("bestmove", 3);
    engine.send("quit");
    engine.wait_end();
    assert_eq!(engine.lines_starting_with("bestmove").len(), 3);
}

#[test]
fn end_of_input_ends_run() {
    let mut engine = Engine::start();
    engine.send("isready");
    engine.close();
    engine.wait_end();
    assert_eq!(engine.lines(), ["readyok"]);
}

#[test]
fn invalid_fen_keeps_the_previous_position() {
    let board = Board::from_fen(ONE_MOVE).unwrap();
    assert_eq!(legal_moves(&board).len(), 1);

    let mut engine = Engine::start();
    engine.send(&format!("position fen {ONE_MOVE}"));
    engine.send("position fen 8/8/8 w - - 0 1");
    engine.wait_for("info string invalid FEN");
    engine.send("go depth 1");
    assert_eq!(engine.wait_for("bestmove"), "bestmove a1b2");
    engine.send("quit");
    engine.wait_end();
}

#[test]
fn impossible_material_is_rejected_instead_of_crashing() {
    // 263 legal moves, more than a move list holds: it used to abort.
    let mut engine = Engine::start();
    engine.send("position fen QQQQQQnk/Q4Qnn/Q5QQ/Q6Q/Q6Q/Q6Q/Q6Q/KQQQQQQQ w - - 0 1");
    engine.wait_for("info string invalid FEN");
    engine.send("go depth 1");
    assert_legal(&Board::startpos(), &engine.wait_for("bestmove"));
    engine.send("quit");
    engine.wait_end();
}

#[test]
fn illegal_move_keeps_the_moves_before_it() {
    let mut engine = Engine::start();
    engine.send(&format!("position fen {ONE_MOVE} moves e2e4"));
    engine.wait_for("info string illegal move e2e4");
    engine.send("go depth 1");
    assert_eq!(engine.wait_for("bestmove"), "bestmove a1b2");

    engine.send("position startpos moves e2e4 e2e4 e7e5");
    engine.wait_for_count("info string illegal move e2e4", 2);
    engine.send("go depth 1");
    let line = engine.wait_for_count("bestmove", 2).remove(1);
    let start = Board::startpos();
    let after_e4 = start.make_move(from_uci(&start, "e2e4", false).unwrap());
    assert_legal(&after_e4, &line);
    engine.send("quit");
    engine.wait_end();
}

#[test]
fn no_legal_move_gives_0000() {
    assert!(legal_moves(&Board::from_fen(MATED).unwrap()).is_empty());
    let mut engine = Engine::start();
    engine.send(&format!("position fen {MATED}"));
    engine.send("go depth 1");
    assert_eq!(engine.wait_for("bestmove"), "bestmove 0000");
    engine.send("quit");
    engine.wait_end();
}

#[test]
fn unknown_and_malformed_lines_are_ignored() {
    let mut engine = Engine::start();
    for line in [
        "",
        "foo bar",
        "position fen",
        "go depth x",
        "setoption",
        "perft",
    ] {
        engine.send(line);
    }
    engine.send("isready");
    engine.wait_for("readyok");
    engine.send("quit");
    engine.wait_end();
}
