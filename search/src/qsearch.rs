use engine::movegen::{All, Context, Noisy, generate};
use engine::{Board, Move, MoveList, PieceType};
use eval::Score;

use crate::shared::Shared;
use crate::thread::ThreadState;

/// Searches captures and queen promotions until the position is quiet. The
/// side to move may stand pat on the static evaluation, except in check,
/// where every evasion is searched. Fail-soft.
pub(crate) fn qsearch(
    t: &mut ThreadState,
    shared: &Shared,
    mut alpha: Score,
    beta: Score,
    ply: usize,
) -> Score {
    t.nodes += 1;
    t.seldepth = t.seldepth.max(ply as u32);
    t.pv.clear_ply(ply);
    t.check_stop(shared);
    if t.stopped {
        // Ignored by the caller.
        return Score::DRAW;
    }
    let board = t.position.board();
    if ply >= Score::MAX_PLY - 1 {
        return t.evaluator.evaluate(board);
    }

    let in_check = board.in_check();
    let mut best = -Score::INFINITE;
    if !in_check {
        best = t.evaluator.evaluate(board);
        if best >= beta {
            return best;
        }
        alpha = alpha.max(best);
    }

    let ctx = Context::new(board);
    let mut moves = MoveList::new();
    if in_check {
        generate::<All>(&ctx, &mut moves);
    } else {
        generate::<Noisy>(&ctx, &mut moves);
    }
    sort_captures(board, moves.as_mut_slice());

    for &mv in moves.as_slice() {
        t.position.make_move(mv);
        let score = -qsearch(t, shared, -beta, -alpha, ply + 1);
        t.position.unmake();
        if t.stopped {
            return Score::DRAW;
        }
        if score > best {
            best = score;
            if score > alpha {
                alpha = score;
                t.pv.update(ply, mv);
                if score >= beta {
                    return best;
                }
            }
        }
    }
    if in_check && moves.is_empty() {
        return Score::mated_in(ply);
    }
    best
}

/// Move ordering values, indexed by `PieceType::index()`. Moves are legal,
/// so a king never captures a defended piece: as an attacker, it costs
/// nothing.
const ORDER_VALUES: [i32; PieceType::COUNT] = [1, 3, 3, 5, 9, 0];

/// Material won first, then the cheapest attacker. A promotion wins the new
/// piece minus the pawn. A castle (the king onto its own rook) wins nothing.
fn mvv_lva(board: &Board, mv: Move) -> i32 {
    let value = |piece: PieceType| ORDER_VALUES[piece.index()];
    let victim = if mv.is_en_passant() {
        Some(PieceType::Pawn)
    } else if mv.is_capture() {
        board.piece_on(mv.to()).map(|p| p.piece_type())
    } else {
        None
    };
    let mut gain = victim.map_or(0, value);
    if let Some(promoted) = mv.promotion_piece() {
        gain += value(promoted) - value(PieceType::Pawn);
    }
    let attacker = board
        .piece_on(mv.from())
        .map_or(0, |p| value(p.piece_type()));
    16 * gain - attacker
}

/// Best `mvv_lva` first. Insertion sort: the lists are short, nothing is
/// allocated, and ties keep the generation order.
fn sort_captures(board: &Board, moves: &mut [Move]) {
    for i in 1..moves.len() {
        let mv = moves[i];
        let key = mvv_lva(board, mv);
        let mut j = i;
        while j > 0 && mvv_lva(board, moves[j - 1]) < key {
            moves[j] = moves[j - 1];
            j -= 1;
        }
        moves[j] = mv;
    }
}

#[cfg(test)]
mod tests {
    use engine::{CastleSide, Position, Square};

    use super::*;

    /// The knight on e3 can take the queen on d5.
    const HANGING_QUEEN: &str = "4k3/8/8/3q4/8/4N3/8/4K3 w - - 0 1";
    /// Qxd5 is the only capture, and c6xd5 takes the queen back.
    const LOSING_CAPTURE: &str = "4k3/8/2p5/3p4/8/8/3Q4/4K3 w - - 0 1";
    /// In check, Kg2 is the only move, then ...Rxa4 wins the knight.
    const KNIGHT_LOST: &str = "4k3/8/8/8/N7/8/7P/r6K w - - 0 1";

    fn state(fen: &str) -> Box<ThreadState> {
        let mut t = Box::new(ThreadState::new());
        t.position = Position::new(Board::from_fen(fen).unwrap());
        t
    }

    fn search(t: &mut ThreadState, ply: usize) -> Score {
        qsearch(t, &Shared::new(), -Score::INFINITE, Score::INFINITE, ply)
    }

    fn search_window(t: &mut ThreadState, alpha: Score, beta: Score) -> Score {
        qsearch(t, &Shared::new(), alpha, beta, 0)
    }

    fn static_eval(t: &mut ThreadState) -> Score {
        t.evaluator.evaluate(t.position.board())
    }

    #[test]
    fn captures_are_sorted_by_mvv_lva() {
        // exd5 (P takes Q), Rxd5 (R takes Q), axb5 (P takes N), Kxf3 (the
        // king costs nothing), Rxf3 (R takes P).
        let board = Board::from_fen("4k3/8/8/1n1q4/P3P3/5pK1/3R4/5R2 w - - 0 1").unwrap();
        let mut moves = MoveList::new();
        generate::<Noisy>(&Context::new(&board), &mut moves);
        sort_captures(&board, moves.as_mut_slice());
        let capture = Move::capture;
        assert_eq!(
            moves.as_slice(),
            [
                capture(Square::E4, Square::D5),
                capture(Square::D2, Square::D5),
                capture(Square::A4, Square::B5),
                capture(Square::G3, Square::F3),
                capture(Square::F1, Square::F3),
            ]
        );
    }

    #[test]
    fn promotions_and_en_passant_are_sorted() {
        // bxa8 (R plus a promotion), b8=Q, exd6 en passant (P takes P),
        // then Rxh5 (R takes P).
        let board = Board::from_fen("r3k3/1P6/8/3pP2p/8/8/8/4K2R w - d6 0 1").unwrap();
        let mut moves = MoveList::new();
        generate::<Noisy>(&Context::new(&board), &mut moves);
        sort_captures(&board, moves.as_mut_slice());
        let takes = |piece| Move::promotion(Square::B7, Square::A8, piece, true);
        assert_eq!(
            moves.as_slice(),
            [
                takes(PieceType::Queen),
                takes(PieceType::Rook),
                Move::promotion(Square::B7, Square::B8, PieceType::Queen, false),
                // Same key: generation order is kept.
                takes(PieceType::Knight),
                takes(PieceType::Bishop),
                Move::en_passant(Square::E5, Square::D6),
                Move::capture(Square::H1, Square::H5),
            ]
        );
    }

    #[test]
    fn castling_is_not_a_capture() {
        let board = Board::from_fen("4k3/8/8/8/8/8/8/4K2R w K - 0 1").unwrap();
        let castle = Move::castle(Square::E1, Square::H1, CastleSide::King);
        assert_eq!(mvv_lva(&board, castle), 0);
    }

    #[test]
    fn in_check_captures_are_searched_first() {
        // Kxe2 is generated after Kd1 and Kf1. Searched first, it cuts at once.
        let mut t = state("4k3/8/8/8/8/8/4r3/4K3 w - - 0 1");
        let beta = Score::new(-200);
        assert!(search_window(&mut t, -Score::INFINITE, beta) >= beta);
        assert_eq!(t.nodes, 2);
    }

    #[test]
    fn quiet_position_is_the_static_eval() {
        let mut t = Box::new(ThreadState::new());
        let eval = static_eval(&mut t);
        assert_eq!(search(&mut t, 0), eval);
        assert!(t.pv.line().is_empty());
    }

    #[test]
    fn hanging_queen_is_taken() {
        let mut t = state(HANGING_QUEEN);
        let eval = static_eval(&mut t);
        assert!(search(&mut t, 0) >= eval + 800);
        assert_eq!(t.pv.line(), [Move::capture(Square::E3, Square::D5)]);
    }

    #[test]
    fn losing_capture_is_refused() {
        let mut t = state(LOSING_CAPTURE);
        let eval = static_eval(&mut t);
        assert_eq!(search(&mut t, 0), eval);
        assert!(t.pv.line().is_empty());
        // The root, Qxd5 and c6xd5: the capture was searched.
        assert_eq!(t.nodes, 3);
    }

    #[test]
    fn stand_pat_cutoff_is_fail_soft() {
        let mut t = state(HANGING_QUEEN);
        let eval = static_eval(&mut t);
        assert_eq!(search_window(&mut t, -Score::INFINITE, eval - 1), eval);
        // Cut before any move is searched.
        assert_eq!(t.nodes, 1);
    }

    #[test]
    fn result_agrees_with_the_full_window() {
        let fens = [
            HANGING_QUEEN,
            LOSING_CAPTURE,
            KNIGHT_LOST,
            // A short exchange on d5 and e4.
            "4k3/8/3r4/3p4/4P3/2N5/8/3RK3 w - - 0 1",
        ];
        for fen in fens {
            let mut t = state(fen);
            let r = search(&mut t, 0);
            let pv = t.pv.line().to_vec();

            let mut t = state(fen);
            assert_eq!(search_window(&mut t, r - 50, r + 50), r, "{fen}");
            assert_eq!(t.pv.line(), pv, "{fen}");

            let mut t = state(fen);
            assert!(search_window(&mut t, r - 200, r - 100) >= r - 100, "{fen}");

            let mut t = state(fen);
            assert!(search_window(&mut t, r + 100, r + 200) <= r + 100, "{fen}");
            // No move raised alpha.
            assert!(t.pv.line().is_empty(), "{fen}");
        }
    }

    #[test]
    fn mate_is_found_in_check() {
        let mut t = state("7k/6Q1/6K1/8/8/8/8/8 b - - 0 1");
        assert_eq!(search(&mut t, 3), Score::mated_in(3));
    }

    #[test]
    fn quiet_evasion_is_searched_in_check() {
        // Kg2 is the only move, and it is not a capture.
        let mut t = state("4k3/8/8/8/8/8/7P/r6K w - - 0 1");
        assert!(!search(&mut t, 0).is_mate());
    }

    #[test]
    fn no_stand_pat_in_check() {
        let mut t = state(KNIGHT_LOST);
        let eval = static_eval(&mut t);
        assert!(search(&mut t, 0) < eval);
        assert_eq!(
            t.pv.line(),
            [
                Move::quiet(Square::H1, Square::G2),
                Move::capture(Square::A1, Square::A4)
            ]
        );
    }

    #[test]
    fn every_call_counts_a_node() {
        let mut t = Box::new(ThreadState::new());
        let _ = search(&mut t, 0);
        assert_eq!(t.nodes, 1);
        assert_eq!(t.seldepth, 0);

        let mut t = state(HANGING_QUEEN);
        let _ = search(&mut t, 0);
        assert!(t.nodes >= 2);
        assert!(t.seldepth >= 1);
    }

    #[test]
    fn deepest_ply_returns_the_static_eval() {
        let mut t = state(HANGING_QUEEN);
        let eval = static_eval(&mut t);
        assert_eq!(search(&mut t, Score::MAX_PLY - 1), eval);
    }

    #[test]
    fn stopped_search_returns_at_once() {
        let mut t = state(HANGING_QUEEN);
        t.can_stop = true;
        t.node_limit = Some(1);
        assert_eq!(search(&mut t, 0), Score::DRAW);
        assert!(t.stopped);
        assert_eq!(t.nodes, 1);
    }

    #[test]
    fn stop_in_a_child_is_not_a_score() {
        let mut t = state(HANGING_QUEEN);
        t.can_stop = true;
        t.node_limit = Some(2);
        assert_eq!(search(&mut t, 0), Score::DRAW);
        assert!(t.stopped);
        assert_eq!(t.nodes, 2);
        assert!(t.pv.line().is_empty());
    }
}
