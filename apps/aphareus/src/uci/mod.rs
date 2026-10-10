pub mod command;

use std::io::{BufRead, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use engine::movegen::legal_moves;
use engine::{Board, CastleSide, Color, File, Move, Position, from_uci, to_uci};

use command::{Command, GoParams, PositionBase, parse};

/// The UCI output, shared by the main loop and the search thread.
type Output = Arc<Mutex<dyn Write + Send>>;

/// Stack of the search thread. Only reserved: memory is used as the
/// search goes deeper.
const SEARCH_STACK_SIZE: usize = 64 << 20;

/// Runs the UCI protocol: reads commands from `input` until `quit` or the
/// end of the input, and writes the answers to `output`.
pub fn run(input: impl BufRead + Send + 'static, output: impl Write + Send + 'static) {
    let output: Output = Arc::new(Mutex::new(output));
    let (lines, received) = mpsc::channel();
    // Detached: it ends with the input, or at its next line once `run` has
    // returned.
    thread::spawn(move || read_lines(input, &lines));
    let mut session = Session::new();
    for line in received {
        match parse(&line) {
            Command::Quit => break,
            command => session.handle(command, &output),
        }
    }
    session.stop_search();
}

/// Sends each line of `input` to the main loop, then `quit` at the end of
/// the input or on a read error.
fn read_lines(mut input: impl BufRead, lines: &Sender<String>) {
    let mut bytes = Vec::new();
    loop {
        bytes.clear();
        match input.read_until(b'\n', &mut bytes) {
            Ok(0) | Err(_) => break,
            Ok(_) => {
                // Lossy: a line that is not UTF-8 becomes an unknown command.
                let line = String::from_utf8_lossy(&bytes);
                let line = line.trim_end_matches(['\r', '\n']).to_string();
                if lines.send(line).is_err() {
                    // `run` has returned.
                    return;
                }
            }
        }
    }
    let _ = lines.send("quit".to_string());
}

/// What the UCI loop keeps between commands.
struct Session {
    position: Position,
    /// Castles are written king takes rook (`e1h1`) instead of `e1g1`.
    chess960: bool,
    /// Raised to end the running search.
    stop: Arc<AtomicBool>,
    /// The search thread, from `go` until it is joined.
    search: Option<JoinHandle<()>>,
}

impl Session {
    fn new() -> Session {
        Session {
            position: Position::new(Board::startpos()),
            chess960: false,
            stop: Arc::new(AtomicBool::new(false)),
            search: None,
        }
    }

    fn handle(&mut self, command: Command, output: &Output) {
        match command {
            Command::Uci => {
                let version = env!("CARGO_PKG_VERSION");
                send(output, &format!("id name Aphareus {version}"));
                send(output, "id author Mathias Gallard");
                send(output, "uciok");
            }
            Command::IsReady => send(output, "readyok"),
            Command::UciNewGame => {
                self.stop_search();
                self.position = Position::new(Board::startpos());
                self.chess960 = false;
            }
            Command::Position { base, moves } => self.set_position(base, &moves, output),
            Command::Go(params) => self.go(params, output),
            Command::Stop => self.stop_search(),
            // `setoption` comes in step 2.3, `perft` and `d` in step 2.4.
            Command::SetOption { .. } | Command::Perft(_) | Command::Display => {}
            // `quit` is handled by `run`.
            Command::Unknown(_) | Command::Quit => {}
        }
    }

    /// An invalid FEN keeps the previous position. An illegal move keeps the
    /// moves before it.
    fn set_position(&mut self, base: PositionBase, moves: &[String], output: &Output) {
        let board = match base {
            PositionBase::StartPos => Board::startpos(),
            PositionBase::Fen(fen) => match Board::from_fen(&fen) {
                Ok(board) => board,
                Err(e) => {
                    send(output, &format!("info string invalid FEN ({e}): {fen}"));
                    return;
                }
            },
        };
        let chess960 = is_chess960(&board);
        let mut position = Position::new(board);
        for text in moves {
            let Some(m) = from_uci(position.board(), text, chess960) else {
                send(
                    output,
                    &format!("info string illegal move {text}: it and the next moves are ignored"),
                );
                break;
            };
            position.make_move(m);
        }
        self.position = position;
        self.chess960 = chess960;
    }

    /// Searches the current position in a thread of its own.
    fn go(&mut self, params: GoParams, output: &Output) {
        // One search at a time: the GUI should have sent `stop`.
        self.stop_search();
        self.stop.store(false, Ordering::Relaxed);
        let position = self.position.clone();
        let chess960 = self.chess960;
        let stop = Arc::clone(&self.stop);
        let output = Arc::clone(output);
        let search = thread::Builder::new()
            .name("search".to_string())
            .stack_size(SEARCH_STACK_SIZE)
            .spawn(move || play_first_move(position, params, chess960, stop, output))
            .expect("the search thread starts");
        self.search = Some(search);
    }

    /// Stops the running search, if any, and waits for its `bestmove`.
    fn stop_search(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(search) = self.search.take() {
            // A search that panicked has nothing left to say.
            let _ = search.join();
        }
    }
}

/// Stand-in for the random mover of step 2.5: plays the first legal move,
/// or `0000` without one. Answers at once, except for `go infinite`, which
/// waits for `stop`.
fn play_first_move(
    position: Position,
    params: GoParams,
    chess960: bool,
    stop: Arc<AtomicBool>,
    output: Output,
) {
    if params.infinite {
        while !stop.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_millis(1));
        }
    }
    let board = position.board();
    let best = legal_moves(board)
        .as_slice()
        .first()
        .copied()
        .unwrap_or(Move::NULL);
    send(
        &output,
        &format!("bestmove {}", to_uci(board, best, chess960)),
    );
}

/// Returns `true` if a castling right of `board` only exists in Chess960:
/// its rook is not in the corner, or its king is not on the e-file.
/// Without castling rights, both notations are the same.
fn is_chess960(board: &Board) -> bool {
    let rights = board.castling_rights();
    Color::ALL.into_iter().any(|color| {
        CastleSide::ALL.into_iter().any(|side| {
            let corner = match side {
                CastleSide::King => File::H,
                CastleSide::Queen => File::A,
            };
            let rook = board.castling_config().rook_start(color, side);
            rights.has(color, side)
                && (board.king_square(color).file() != File::E
                    || rook.map(|sq| sq.file()) != Some(corner))
        })
    })
}

/// Writes `line` and flushes it: the GUI reads line by line.
fn send(output: &Output, line: &str) {
    // A writer poisoned by a panic still holds whole lines. A closed output
    // cannot be reported anywhere; the end of the input will follow.
    let mut output = output.lock().unwrap_or_else(PoisonError::into_inner);
    let _ = writeln!(output, "{line}");
    let _ = output.flush();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chess960(fen: &str) -> bool {
        is_chess960(&Board::from_fen(fen).unwrap())
    }

    #[test]
    fn standard_castling_is_not_chess960() {
        assert!(!chess960(Board::STARTPOS_FEN));
        assert!(!chess960("4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1"));
    }

    #[test]
    fn rook_off_its_corner_is_chess960() {
        assert!(chess960("4k3/8/8/8/8/8/8/R4KR1 w GA - 0 1"));
        assert!(chess960("4k3/8/8/8/8/8/8/4K1R1 w K - 0 1"));
    }

    #[test]
    fn king_off_the_e_file_is_chess960() {
        assert!(chess960("4k3/8/8/8/8/8/8/R2K3R w HA - 0 1"));
        assert!(chess960("r2k3r/8/8/8/8/8/8/4K3 w ha - 0 1"));
    }

    #[test]
    fn no_castling_right_is_not_chess960() {
        assert!(!chess960("4k3/8/8/8/8/8/8/R2K2R1 w - - 0 1"));
    }
}
