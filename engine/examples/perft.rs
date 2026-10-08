//! Runs perft and prints node counts, time and speed.
//!
//! ```text
//! cargo run --release --example perft -- <depth> [FEN]           # depths 1..=depth
//! cargo run --release --example perft -- <depth> [FEN] --divide  # per move
//! cargo run --release --example perft -- --suite <file.epd> [max depth]
//! ```
//!
//! Without a FEN, the starting position is used. Leaves are counted without
//! being played (bulk counting), so the speed is in leaves per second, far
//! above the moves actually played.

use std::process::ExitCode;
use std::time::{Duration, Instant};
use std::{env, fs};

use engine::{Board, divide, perft, to_uci};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("--suite") => run_suite(&args[1..]),
        Some(_) => run_position(&args),
        None => Err(usage()),
    };
    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn usage() -> String {
    "usage: perft <depth> [FEN] [--divide [--chess960]]\n       perft --suite <file.epd> [max depth]"
        .into()
}

fn parse_depth(arg: &str) -> Result<u32, String> {
    arg.parse()
        .map_err(|_| format!("invalid depth {arg:?}\n{}", usage()))
}

/// Leaves per second, in millions.
fn mnps(nodes: u64, elapsed: Duration) -> f64 {
    nodes as f64 / elapsed.as_secs_f64().max(1e-9) / 1e6
}

fn run_position(args: &[String]) -> Result<bool, String> {
    let depth = parse_depth(&args[0])?;
    let divide_mode = args.iter().any(|a| a == "--divide");
    let chess960 = args.iter().any(|a| a == "--chess960");
    let fen: Vec<&str> = args[1..]
        .iter()
        .map(String::as_str)
        .filter(|a| !a.starts_with("--"))
        .collect();
    let board = if fen.is_empty() {
        Board::startpos()
    } else {
        Board::from_fen(&fen.join(" ")).map_err(|e| format!("invalid FEN: {e}"))?
    };
    println!("{board}");

    if divide_mode {
        let start = Instant::now();
        let split = divide(&board, depth.max(1));
        let elapsed = start.elapsed();
        for &(m, nodes) in &split {
            println!("{}: {nodes}", to_uci(&board, m, chess960));
        }
        let total: u64 = split.iter().map(|&(_, n)| n).sum();
        println!(
            "\n{} moves, {total} nodes in {:.3} s ({:.1} Mnps)",
            split.len(),
            elapsed.as_secs_f64(),
            mnps(total, elapsed)
        );
        return Ok(true);
    }

    println!(
        "{:>5} {:>15} {:>10} {:>10}",
        "depth", "nodes", "time (s)", "Mnps"
    );
    for d in 1..=depth {
        let start = Instant::now();
        let nodes = perft(&board, d);
        let elapsed = start.elapsed();
        println!(
            "{d:>5} {nodes:>15} {:>10.3} {:>10.1}",
            elapsed.as_secs_f64(),
            mnps(nodes, elapsed)
        );
    }
    Ok(true)
}

fn run_suite(args: &[String]) -> Result<bool, String> {
    let path = args.first().ok_or_else(usage)?;
    let max_depth = args
        .get(1)
        .map(|a| parse_depth(a))
        .transpose()?
        .unwrap_or(u32::MAX);
    let text = fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;

    let (mut total_nodes, mut total_time, mut failures) = (0, Duration::ZERO, 0);
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let mut fields = line.split(';');
        let fen = fields.next().unwrap_or_default().trim();
        let board = Board::from_fen(fen).map_err(|e| format!("{fen}: {e}"))?;
        println!("{fen}");
        for field in fields {
            let Some((depth, expected)) = field
                .trim()
                .strip_prefix('D')
                .and_then(|f| f.split_once(' '))
            else {
                return Err(format!("malformed field {field:?}"));
            };
            let depth = parse_depth(depth)?;
            if depth > max_depth {
                break;
            }
            let expected: u64 = expected
                .parse()
                .map_err(|_| format!("bad count {expected:?}"))?;
            let start = Instant::now();
            let nodes = perft(&board, depth);
            let elapsed = start.elapsed();
            let status = if nodes == expected { "ok" } else { "FAIL" };
            if nodes != expected {
                failures += 1;
            }
            println!(
                "  D{depth} {nodes:>13} {:>9.3} s {:>7.1} Mnps  {status}{}",
                elapsed.as_secs_f64(),
                mnps(nodes, elapsed),
                if nodes == expected {
                    String::new()
                } else {
                    format!(" (expected {expected})")
                }
            );
            total_nodes += nodes;
            total_time += elapsed;
        }
    }
    println!(
        "\n{total_nodes} nodes in {:.3} s ({:.1} Mnps), {failures} failure(s)",
        total_time.as_secs_f64(),
        mnps(total_nodes, total_time)
    );
    Ok(failures == 0)
}
