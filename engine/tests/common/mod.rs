#![allow(dead_code)]

use engine::Board;

pub const STANDARD: &str = include_str!("../data/perft_standard.epd");
pub const CHESS960: &str = include_str!("../data/perft_960.epd");

/// One EPD line: a position and its known perft counts.
pub struct PerftCase {
    pub fen: String,
    pub board: Board,
    /// `(depth, nodes)`, by increasing depth.
    pub counts: Vec<(u32, u64)>,
}

/// Reads lines of the form `FEN ;D1 n1 ;D2 n2 …`.
pub fn load(text: &str) -> Vec<PerftCase> {
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let mut fields = line.split(';');
            let fen = fields.next().unwrap().trim().to_string();
            let board = Board::from_fen(&fen).unwrap_or_else(|e| panic!("{fen}: {e}"));
            let counts = fields
                .map(|field| {
                    let (depth, nodes) = field
                        .trim()
                        .strip_prefix('D')
                        .unwrap()
                        .split_once(' ')
                        .unwrap();
                    (depth.parse().unwrap(), nodes.parse().unwrap())
                })
                .collect();
            PerftCase { fen, board, counts }
        })
        .collect()
}

/// Every position of both suites.
pub fn all_cases() -> Vec<PerftCase> {
    let mut cases = load(STANDARD);
    cases.extend(load(CHESS960));
    cases
}
