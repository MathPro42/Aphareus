use std::fs;
use std::io::Write;
use std::time::Instant;

use engine::{Board, perft};

use crate::uci::extra::nps;

/// Runs the command-line mode. `args` are the arguments without the program
/// name. Returns the process exit code: 0 on success, 1 on failure.
pub fn run(args: &[String], out: &mut impl Write, err: &mut impl Write) -> u8 {
    match args.first().map(String::as_str) {
        Some("perft") => perft_suite(args.get(1), out, err),
        // An error message that cannot be written has nowhere else to go.
        Some(command) => {
            let _ = writeln!(err, "unknown command: {command}");
            1
        }
        None => {
            let _ = writeln!(err, "missing command");
            1
        }
    }
}

/// `perft <file>`: checks every count of an EPD file.
fn perft_suite(path: Option<&String>, out: &mut impl Write, err: &mut impl Write) -> u8 {
    let Some(path) = path else {
        let _ = writeln!(err, "usage: aphareus perft <file.epd>");
        return 1;
    };
    match fs::read_to_string(path) {
        Ok(text) => check_suite(&text, out, err),
        Err(e) => {
            let _ = writeln!(err, "cannot read {path}: {e}");
            1
        }
    }
}

/// Checks lines of the form `FEN ;D1 n1 ;D2 n2 …`, one output line per
/// count. Blank lines and fields are skipped. A bad line or count is
/// reported on `err` and the next ones still run.
/// Returns 1 if anything failed.
fn check_suite(text: &str, out: &mut impl Write, err: &mut impl Write) -> u8 {
    let mut failed = false;
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let mut fields = line.split(';');
        let fen = fields.next().unwrap_or_default().trim();
        let board = match Board::from_fen(fen) {
            Ok(board) => board,
            Err(e) => {
                let _ = writeln!(err, "invalid FEN ({e}): {fen}");
                failed = true;
                continue;
            }
        };
        for field in fields.filter(|field| !field.trim().is_empty()) {
            let Some((depth, expected)) = parse_count(field) else {
                let _ = writeln!(err, "invalid count {:?}: {fen}", field.trim());
                failed = true;
                continue;
            };
            let start = Instant::now();
            let nodes = perft(&board, depth);
            let elapsed = start.elapsed();
            let status = if nodes == expected { "OK" } else { "FAIL" };
            failed |= nodes != expected;
            let _ = writeln!(
                out,
                "{status}  D{depth}  expected={expected}  got={nodes}  {} ms  {} nps  {fen}",
                elapsed.as_millis(),
                nps(nodes, elapsed)
            );
        }
    }
    u8::from(failed)
}

/// `D<depth> <nodes>` as `(depth, nodes)`.
fn parse_count(field: &str) -> Option<(u32, u64)> {
    let mut tokens = field.split_whitespace();
    let depth = tokens.next()?.strip_prefix('D')?.parse().ok()?;
    let nodes = tokens.next()?.parse().ok()?;
    tokens.next().is_none().then_some((depth, nodes))
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINI: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/perft_mini.epd");

    const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    /// `(exit code, stdout, stderr)` of `run` on `args`.
    fn run_with(args: &[&str]) -> (u8, String, String) {
        let args: Vec<String> = args.iter().map(|&arg| arg.to_string()).collect();
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = run(&args, &mut out, &mut err);
        (
            code,
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
    }

    /// `(exit code, stdout, stderr)` of `check_suite` on `text`.
    fn check_with(text: &str) -> (u8, String, String) {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = check_suite(text, &mut out, &mut err);
        (
            code,
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
    }

    /// The status of each output line.
    fn statuses(out: &str) -> Vec<&str> {
        out.lines()
            .map(|line| line.split_whitespace().next().unwrap())
            .collect()
    }

    #[test]
    fn unknown_command_fails() {
        let (code, out, err) = run_with(&["inconnue", "x"]);
        assert_eq!(code, 1);
        assert!(out.is_empty());
        assert_eq!(err, "unknown command: inconnue\n");
    }

    #[test]
    fn missing_command_fails() {
        let (code, out, err) = run_with(&[]);
        assert_eq!(code, 1);
        assert!(out.is_empty());
        assert_eq!(err, "missing command\n");
    }

    #[test]
    fn perft_file_passes() {
        let (code, out, err) = run_with(&["perft", MINI]);
        assert_eq!(code, 0, "{out}{err}");
        assert_eq!(statuses(&out), ["OK"; 4]);
        assert!(err.is_empty(), "{err}");
        let first = out.lines().next().unwrap();
        assert!(
            first.starts_with("OK  D1  expected=20  got=20  "),
            "{first}"
        );
        assert!(first.ends_with(&format!(" nps  {START_FEN}")), "{first}");
    }

    #[test]
    fn wrong_count_fails_only_its_line() {
        let text = format!("{START_FEN} ;D1 20 ;D2 401 ;D3 8902\n");
        let (code, out, err) = check_with(&text);
        assert_eq!(code, 1);
        assert_eq!(statuses(&out), ["OK", "FAIL", "OK"]);
        assert!(out.contains("FAIL  D2  expected=401  got=400  "), "{out}");
        assert!(err.is_empty(), "{err}");
    }

    #[test]
    fn invalid_fen_fails_and_the_next_lines_run() {
        let text = format!("8/8/8 w - - 0 1 ;D1 1\n\n{START_FEN} ;D1 20\n");
        let (code, out, err) = check_with(&text);
        assert_eq!(code, 1);
        assert_eq!(statuses(&out), ["OK"]);
        assert!(err.starts_with("invalid FEN ("), "{err}");
        assert!(err.ends_with("): 8/8/8 w - - 0 1\n"), "{err}");
    }

    #[test]
    fn invalid_count_fails_and_the_next_counts_run() {
        for field in ["D2", "2 400", "Dx 400", "D2 x", "D2 400 1"] {
            let text = format!("{START_FEN} ;{field} ;D1 20\n");
            let (code, out, err) = check_with(&text);
            assert_eq!(code, 1, "{field:?}");
            assert_eq!(statuses(&out), ["OK"], "{field:?}");
            assert_eq!(err, format!("invalid count {field:?}: {START_FEN}\n"));
        }
    }

    #[test]
    fn blank_fields_are_skipped() {
        let (code, out, err) = check_with(&format!("{START_FEN} ;D1 20 ; ;\n"));
        assert_eq!(code, 0, "{err}");
        assert_eq!(statuses(&out), ["OK"]);
    }

    #[test]
    fn perft_without_file_fails() {
        let (code, out, err) = run_with(&["perft"]);
        assert_eq!(code, 1);
        assert!(out.is_empty());
        assert_eq!(err, "usage: aphareus perft <file.epd>\n");
    }

    #[test]
    fn unreadable_file_fails() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/missing.epd");
        let (code, out, err) = run_with(&["perft", path]);
        assert_eq!(code, 1);
        assert!(out.is_empty());
        assert!(err.starts_with(&format!("cannot read {path}: ")), "{err}");
    }
}
