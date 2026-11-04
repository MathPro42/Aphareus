pub mod command;
pub(crate) mod extra;
mod options;
mod output;

use std::io::{BufRead, Write};
use std::sync::{Arc, Mutex};

use engine::{Board, CastleSide, Color, File, Position, from_uci};
use search::{Limits, SearchPool};

use command::{Command, PositionBase, parse};
use options::{DECLARATIONS, Options};
use output::{Output, UciReporter, send};

/// Runs the UCI protocol: reads commands from `input` until `quit` or the
/// end of the input, and writes the answers to `output`.
pub fn run(mut input: impl BufRead, output: impl Write + Send + 'static) {
    let output: Output = Arc::new(Mutex::new(output));
    let mut session = Session::new();
    let mut bytes = Vec::new();
    loop {
        bytes.clear();
        // The end of the input, or a read error, is `quit`.
        match input.read_until(b'\n', &mut bytes) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
        // Lossy: a line that is not UTF-8 becomes an unknown command.
        let line = String::from_utf8_lossy(&bytes);
        match parse(line.trim_end_matches(['\r', '\n'])) {
            Command::Quit => break,
            command => session.handle(command, &output),
        }
    }
    session.pool.quit();
}

/// What the UCI loop keeps between commands.
struct Session {
    position: Position,
    /// Castles are written king takes rook (`e1h1`) instead of `e1g1`.
    chess960: bool,
    options: Options,
    pool: SearchPool,
}

impl Session {
    fn new() -> Session {
        Session {
            position: Position::new(Board::startpos()),
            chess960: false,
            options: Options::default(),
            pool: SearchPool::new(),
        }
    }

    /// `go`, `ucinewgame`, `perft` and `bench` stop the running search and
    /// wait for it; `stop` only asks it to stop. The other commands do not
    /// touch it: the search has its own copy of
    /// the position and reads no option.
    fn handle(&mut self, command: Command, output: &Output) {
        match command {
            Command::Uci => {
                let version = env!("CARGO_PKG_VERSION");
                send(output, &format!("id name Aphareus {version}"));
                send(output, "id author Mathias Gallard");
                for spin in DECLARATIONS {
                    send(output, &spin.to_string());
                }
                send(output, "uciok");
            }
            Command::IsReady => send(output, "readyok"),
            Command::UciNewGame => {
                self.pool.new_game();
                self.position = Position::new(Board::startpos());
                self.chess960 = false;
            }
            Command::Position { base, moves } => self.set_position(base, &moves, output),
            Command::Go(limits) => self.go(limits, output),
            Command::Stop => self.pool.stop(),
            Command::SetOption { name, value } => {
                if let Err(e) = self.options.set(&name, value.as_deref()) {
                    send(output, &format!("info string setoption {name}: {e}"));
                }
            }
            Command::Perft(depth) => {
                self.stop_search();
                send(output, &extra::perft(self.position.board(), depth));
            }
            Command::Display => send(output, &extra::display(self.position.board())),
            Command::Eval => send(output, &extra::eval(self.position.board())),
            Command::Bench(Ok(depth)) => {
                self.stop_search();
                let depth = depth.unwrap_or(search::bench::BENCH_DEPTH);
                send(output, &extra::bench(depth));
            }
            Command::Bench(Err(depth)) => {
                send(output, &format!("info string bench: invalid depth {depth}"));
            }
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

    /// Starts searching the current position and returns at once. A running
    /// search is stopped first.
    fn go(&mut self, limits: Limits, output: &Output) {
        let limits = Limits {
            // Clamped to 0..=5000 by the options.
            move_overhead: self.options.move_overhead.unsigned_abs(),
            ..limits
        };
        let reporter = UciReporter::new(
            self.position.board().clone(),
            self.chess960,
            Arc::clone(output),
        );
        self.pool
            .start(self.position.clone(), limits, Box::new(reporter));
    }

    /// Stops the running search, if any, and waits for its `bestmove`.
    fn stop_search(&mut self) {
        self.pool.stop();
        self.pool.wait();
    }
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
