mod common;

use engine::{Board, Prng};
use eval::{Evaluator, Score};

/// `board` with the other side to move and no en passant square, if that
/// position is legal (the side not to move must not be in check).
fn flipped(board: &Board) -> Option<Board> {
    let fen = board.to_fen();
    let mut fields: Vec<&str> = fen.split_whitespace().collect();
    fields[1] = if fields[1] == "w" { "b" } else { "w" };
    fields[3] = "-";
    Board::from_fen(&fields.join(" ")).ok()
}

#[test]
fn side_to_move_point_of_view() {
    let positions = common::positions(1);
    let mut e = Evaluator::new();
    let mut flips = 0;
    for board in &positions {
        if let Some(other) = flipped(board) {
            assert_eq!(e.evaluate(&other), -e.evaluate(board), "{}", board.to_fen());
            flips += 1;
        }
    }
    // Only the positions where the side to move is in check are skipped.
    assert!(
        flips * 10 >= positions.len() * 9,
        "{flips} of {}",
        positions.len()
    );
}

#[test]
fn no_state_between_calls() {
    let positions = common::positions(1);
    let fresh: Vec<Score> = positions
        .iter()
        .map(|b| Evaluator::new().evaluate(b))
        .collect();

    // One evaluator for every position, in order, then shuffled.
    let mut e = Evaluator::new();
    let mut order: Vec<usize> = (0..positions.len()).collect();
    let mut prng = Prng::new(0x9E37_79B9_7F4A_7C15);
    for i in (1..order.len()).rev() {
        let j = (prng.next_u64() % (i as u64 + 1)) as usize;
        order.swap(i, j);
    }
    for i in (0..positions.len()).chain(order) {
        assert_eq!(
            e.evaluate(&positions[i]),
            fresh[i],
            "{}",
            positions[i].to_fen()
        );
    }
}

#[test]
fn never_a_mate_score() {
    let mut e = Evaluator::new();
    for board in common::positions(1) {
        let s = e.evaluate(&board);
        assert!(
            -Score::EVAL_MAX <= s && s <= Score::EVAL_MAX,
            "{s:?}: {}",
            board.to_fen()
        );
        assert!(!s.is_mate(), "{s:?}: {}", board.to_fen());
    }
}
