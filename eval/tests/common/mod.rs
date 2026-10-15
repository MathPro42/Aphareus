use engine::Board;
use engine::movegen::legal_moves;

const STANDARD: &str = include_str!("../../../engine/tests/data/perft_standard.epd");
const CHESS960: &str = include_str!("../../../engine/tests/data/perft_960.epd");
const POSITIONS: &str = include_str!("../data/positions.txt");

fn parse(fen: &str) -> Board {
    Board::from_fen(fen).unwrap_or_else(|e| panic!("{fen}: {e}"))
}

/// The positions of an EPD file: the FEN before the first `;` of each line.
fn epd(text: &str) -> impl Iterator<Item = Board> {
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| parse(line.split(';').next().unwrap().trim()))
}

/// `board` and its descendants down to `depth` plies.
fn with_descendants(board: Board, depth: u32, out: &mut Vec<Board>) {
    if depth > 0 {
        for &m in &legal_moves(&board) {
            with_descendants(board.make_move(m), depth - 1, out);
        }
    }
    out.push(board);
}

/// The standard perft suite with its descendants to depth 2, the Chess960
/// suite with its descendants to `chess960_depth`, then `positions.txt`.
pub fn positions(chess960_depth: u32) -> Vec<Board> {
    let mut out = Vec::new();
    for board in epd(STANDARD) {
        with_descendants(board, 2, &mut out);
    }
    for board in epd(CHESS960) {
        with_descendants(board, chess960_depth, &mut out);
    }
    let own = POSITIONS
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'));
    out.extend(own.map(parse));
    out
}
