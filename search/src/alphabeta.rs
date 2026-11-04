use engine::MoveList;
use engine::movegen::{Context, Noisy, Quiet, generate, legal_moves};
use eval::Score;

use crate::qsearch::qsearch;
use crate::shared::Shared;
use crate::thread::ThreadState;

/// Negamax alpha-beta, fail-soft: returns the best score found, even outside
/// the window. Scores mates, stalemates and draws. At depth 0, hands over to
/// the quiescence search.
pub(crate) fn alphabeta(
    t: &mut ThreadState,
    shared: &Shared,
    depth: i32,
    mut alpha: Score,
    beta: Score,
    ply: usize,
) -> Score {
    if depth <= 0 {
        return qsearch(t, shared, alpha, beta, ply);
    }
    t.nodes += 1;
    t.seldepth = t.seldepth.max(ply as u32);
    t.pv.clear_ply(ply);
    t.check_stop(shared);
    if t.stopped {
        // Ignored by the caller.
        return Score::DRAW;
    }
    // The root always searches its moves: it must return one.
    if ply > 0 {
        if is_draw(t, ply) {
            return Score::DRAW;
        }
        if ply >= Score::MAX_PLY - 1 {
            let board = t.position.board();
            // A mate stays a mate, even at the deepest ply.
            if board.in_check() && legal_moves(board).is_empty() {
                return Score::mated_in(ply);
            }
            return t.evaluator.evaluate(board);
        }
    }

    let board = t.position.board();
    let in_check = board.in_check();
    // Noisy moves first. One context serves both generations; both run
    // before the loop because the context borrows the board.
    let ctx = Context::new(board);
    let mut moves = MoveList::new();
    generate::<Noisy>(&ctx, &mut moves);
    generate::<Quiet>(&ctx, &mut moves);
    if moves.is_empty() {
        return if in_check {
            Score::mated_in(ply)
        } else {
            Score::DRAW
        };
    }

    let mut best = -Score::INFINITE;
    for &mv in moves.as_slice() {
        t.stack.at(ply).mv = mv;
        t.position.make_move(mv);
        let score = -alphabeta(t, shared, depth - 1, -beta, -alpha, ply + 1);
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
                    break;
                }
            }
        }
    }
    best
}

/// Insufficient material, repetition, or fifty moves. A mate given on the
/// hundredth ply is still a mate.
pub(crate) fn is_draw(t: &ThreadState, ply: usize) -> bool {
    let board = t.position.board();
    if board.is_insufficient_material() || t.position.is_repetition_draw(ply) {
        return true;
    }
    board.halfmove_clock() >= 100 && (!board.in_check() || !legal_moves(board).is_empty())
}

#[cfg(test)]
mod tests {
    use engine::{Board, Move, Position, from_uci, to_uci};

    use super::*;

    const MATES: &str = include_str!("../tests/data/mates.epd");
    const PERFT_STANDARD: &str = include_str!("../../engine/tests/data/perft_standard.epd");
    const PERFT_960: &str = include_str!("../../engine/tests/data/perft_960.epd");

    /// Mate in 1 on the back rank: Ra8.
    const BACK_RANK: &str = "6k1/5ppp/8/8/8/8/8/R5K1 w - - 0 1";
    /// Black to move has no move and is not in check.
    const STALEMATED: &str = "7k/5Q2/6K1/8/8/8/8/8 b - - 0 1";
    /// Qxg6 takes the last black piece and stalemates; Kxg6 wins.
    const STALEMATE_TRAP: &str = "7k/5K2/6r1/8/8/8/8/6Q1 w - - 0 1";
    /// Clock at 99: Qf8 mates on the hundredth ply, any other move draws.
    const FIFTY_MATE: &str = "7k/8/6K1/8/8/8/8/5Q2 w - - 99 80";
    /// Same material, no mate in 1, clock at 99: every move draws.
    const FIFTY_NO_MATE: &str = "7k/8/8/5K2/8/8/8/5Q2 w - - 99 80";
    /// King and knight against king.
    const KNIGHT_ONLY: &str = "8/8/4k3/8/8/3NK3/8/8 w - - 0 1";
    /// White is lost but has a perpetual check: Qh5+ Kg8 Qe8+ Kh7.
    const PERPETUAL: &str = "7k/q5p1/8/7Q/8/8/rr6/7K b - - 0 1";
    /// White is winning and could give the same checks.
    const WINNING: &str = "7k/6p1/8/7Q/8/8/8/1R4K1 b - - 0 1";
    /// Twice through the cycle: the root (Kh7, Qe8) has been seen before,
    /// and Qh5+ Kg8 reaches a third occurrence.
    const CYCLE: &str = "h8g8 h5e8 g8h7 e8h5 h7g8 h5e8 g8h7";

    fn state(fen: &str) -> Box<ThreadState> {
        let mut t = Box::new(ThreadState::new());
        t.position = Position::new(Board::from_fen(fen).unwrap());
        t
    }

    /// `fen`, then `moves` played as in `position fen … moves …`.
    fn played(fen: &str, moves: &str) -> Box<ThreadState> {
        let mut t = state(fen);
        for text in moves.split_whitespace() {
            let mv = uci(&t, text);
            t.position.make_move(mv);
        }
        t
    }

    fn uci(t: &ThreadState, text: &str) -> Move {
        from_uci(t.position.board(), text, false).unwrap()
    }

    fn search(t: &mut ThreadState, depth: i32) -> Score {
        let window = Score::INFINITE;
        alphabeta(t, &Shared::new(), depth, -window, window, 0)
    }

    fn search_window(t: &mut ThreadState, depth: i32, alpha: Score, beta: Score) -> Score {
        alphabeta(t, &Shared::new(), depth, alpha, beta, 0)
    }

    fn first_move(t: &ThreadState) -> Move {
        t.pv.line()[0]
    }

    /// The FENs of an EPD file: the text before the first `;` of each line.
    fn fens(epd: &str) -> impl Iterator<Item = &str> {
        epd.lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.split(';').next().unwrap().trim())
    }

    /// The lines of `mates.epd`: FEN, key moves and mate distance in moves.
    fn mates() -> Vec<(&'static str, Vec<&'static str>, i32)> {
        MATES
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                let fields: Vec<&str> = line.split(';').map(str::trim).collect();
                let keys = fields[1].strip_prefix("bm ").unwrap();
                let moves = fields[2].strip_prefix("dm ").unwrap();
                (
                    fields[0],
                    keys.split_whitespace().collect(),
                    moves.parse().unwrap(),
                )
            })
            .collect()
    }

    #[test]
    fn back_rank_mate_in_one() {
        let mut t = state(BACK_RANK);
        assert_eq!(search(&mut t, 1), Score::mate_in(1));
        assert_eq!(t.pv.line(), [uci(&t, "a1a8")]);
    }

    /// Each line was checked by an exhaustive mate search: no shorter mate,
    /// and `bm` lists every key move.
    #[test]
    fn mates_are_found_at_their_distance() {
        for (fen, keys, n) in mates() {
            let plies = 2 * n - 1;
            let mut t = state(fen);
            assert_eq!(
                search(&mut t, plies),
                Score::mate_in(plies as usize),
                "{fen}"
            );
            let key = to_uci(t.position.board(), first_move(&t), false);
            assert!(keys.contains(&key.as_str()), "{fen}: {key}");
            assert_eq!(t.pv.line().len(), plies as usize, "{fen}");
        }
    }

    #[test]
    fn mated_side_sees_the_distance() {
        for (fen, keys, n) in mates() {
            let plies = 2 * n - 2;
            let mut t = played(fen, keys[0]);
            assert_eq!(
                search(&mut t, plies),
                Score::mated_in(plies as usize),
                "{fen}"
            );
        }
    }

    #[test]
    fn stalemate_at_the_root_is_a_draw() {
        let mut t = state(STALEMATED);
        assert_eq!(search(&mut t, 1), Score::DRAW);
        assert!(t.pv.line().is_empty());
    }

    #[test]
    fn stalemate_in_the_tree_is_a_draw() {
        let mut t = state(STALEMATE_TRAP);
        let take_rook = uci(&t, "g1g6");
        t.position.make_move(take_rook);
        let window = Score::INFINITE;
        let child = alphabeta(&mut t, &Shared::new(), 1, -window, window, 1);
        assert_eq!(child, Score::DRAW);

        let mut t = state(STALEMATE_TRAP);
        assert!(search(&mut t, 2) > Score::new(500));
        assert_ne!(first_move(&t), take_rook);
    }

    #[test]
    fn mate_on_the_hundredth_ply_is_a_mate() {
        let mut t = state(FIFTY_MATE);
        assert_eq!(search(&mut t, 2), Score::mate_in(1));
        assert_eq!(first_move(&t), uci(&t, "f1f8"));

        // Any other move reaches the hundredth ply without mate.
        let mut t = played(FIFTY_MATE, "f1f2");
        let window = Score::INFINITE;
        let child = alphabeta(&mut t, &Shared::new(), 1, -window, window, 1);
        assert_eq!(child, Score::DRAW);
    }

    #[test]
    fn fifty_moves_without_mate_is_a_draw() {
        let mut t = state(FIFTY_NO_MATE);
        assert_eq!(search(&mut t, 2), Score::DRAW);

        // The clock alone makes the draw.
        let mut t = state(&FIFTY_NO_MATE.replace(" 99 ", " 0 "));
        assert!(search(&mut t, 2) > Score::new(500));
    }

    #[test]
    fn insufficient_material_is_a_draw() {
        let mut t = state(KNIGHT_ONLY);
        assert_eq!(search(&mut t, 2), Score::DRAW);
    }

    #[test]
    fn root_is_never_a_draw() {
        // Drawn twice over, but the root still searches and keeps a move.
        let mut t = state(&KNIGHT_ONLY.replace(" 0 1", " 100 80"));
        let _ = search(&mut t, 1);
        assert_eq!(t.pv.line().len(), 1);
    }

    #[test]
    fn losing_side_finds_the_perpetual() {
        let mut t = played(PERPETUAL, CYCLE);
        assert_eq!(search(&mut t, 3), Score::DRAW);
        assert_eq!(first_move(&t), uci(&t, "e8h5"));
    }

    #[test]
    fn repetition_needs_the_game_history() {
        let mut t = played(PERPETUAL, CYCLE);
        assert_eq!(search(&mut t, 2), Score::DRAW);
        let fen = t.position.board().to_fen();
        let mut t = state(&fen);
        assert!(search(&mut t, 2) < Score::DRAW);
    }

    #[test]
    fn repetition_is_seen_before_the_quiescence_search() {
        let mut t = played(PERPETUAL, &format!("{CYCLE} e8h5 h7g8"));
        assert_eq!(search(&mut t, 1), Score::DRAW);
        assert_eq!(t.pv.line(), [uci(&t, "h5e8")]);
    }

    #[test]
    fn winning_side_does_not_repeat() {
        let mut t = played(WINNING, CYCLE);
        assert!(search(&mut t, 3) > Score::new(500));
        assert_ne!(first_move(&t), uci(&t, "e8h5"));
    }

    /// At depth 1, the root is the best `-qsearch` of its children, in
    /// generation order (noisy, then quiet), with the same line.
    fn check_depth_one<'a>(fens: impl Iterator<Item = &'a str>) {
        let window = Score::INFINITE;
        for fen in fens {
            let mut t = state(fen);
            let score = search(&mut t, 1);

            let board = Board::from_fen(fen).unwrap();
            let ctx = Context::new(&board);
            let mut moves = MoveList::new();
            generate::<Noisy>(&ctx, &mut moves);
            generate::<Quiet>(&ctx, &mut moves);
            let mut best = -Score::INFINITE;
            let mut line = Vec::new();
            for &mv in moves.as_slice() {
                let mut child = state(fen);
                child.position.make_move(mv);
                let s = -qsearch(&mut child, &Shared::new(), -window, window, 1);
                if s > best {
                    best = s;
                    child.pv.update(0, mv);
                    line = child.pv.line().to_vec();
                }
            }
            assert_eq!(score, best, "{fen}");
            assert_eq!(t.pv.line(), line, "{fen}");
        }
    }

    #[test]
    fn depth_one_is_the_best_child_qsearch() {
        check_depth_one(fens(PERFT_STANDARD).chain(fens(PERFT_960).take(100)));
    }

    #[test]
    #[ignore = "long: run in release with --ignored"]
    fn depth_one_is_the_best_child_qsearch_full() {
        check_depth_one(fens(PERFT_STANDARD).chain(fens(PERFT_960)));
    }

    #[test]
    fn result_agrees_with_the_full_window() {
        let fens = [
            BACK_RANK,
            STALEMATE_TRAP,
            // Kiwipete.
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        ];
        for fen in fens {
            let mut t = state(fen);
            let r = search(&mut t, 2);
            let pv = t.pv.line().to_vec();

            let mut t = state(fen);
            assert_eq!(search_window(&mut t, 2, r - 50, r + 50), r, "{fen}");
            assert_eq!(t.pv.line(), pv, "{fen}");

            let mut t = state(fen);
            assert!(
                search_window(&mut t, 2, r - 200, r - 100) >= r - 100,
                "{fen}"
            );

            let mut t = state(fen);
            assert!(
                search_window(&mut t, 2, r + 100, r + 200) <= r + 100,
                "{fen}"
            );
            // No move raised alpha.
            assert!(t.pv.line().is_empty(), "{fen}");
        }
    }

    #[test]
    fn depth_zero_is_the_quiescence_search() {
        for depth in [0, -3] {
            let mut t = state(STALEMATE_TRAP);
            let score = search(&mut t, depth);
            let nodes = t.nodes;

            let mut q = state(STALEMATE_TRAP);
            let window = Score::INFINITE;
            assert_eq!(qsearch(&mut q, &Shared::new(), -window, window, 0), score);
            assert_eq!(q.nodes, nodes);
        }
    }

    #[test]
    fn deepest_ply_returns_the_static_eval() {
        // A quiet position, then a check with an escape (Kg2).
        for fen in [STALEMATE_TRAP, "4k3/8/8/8/8/8/7P/r6K w - - 0 1"] {
            let mut t = state(fen);
            let eval = t.evaluator.evaluate(t.position.board());
            let window = Score::INFINITE;
            let ply = Score::MAX_PLY - 1;
            let score = alphabeta(&mut t, &Shared::new(), 3, -window, window, ply);
            assert_eq!(score, eval, "{fen}");
        }
    }

    #[test]
    fn deepest_ply_still_scores_a_mate() {
        let mut t = state("7k/6Q1/6K1/8/8/8/8/8 b - - 0 1");
        let window = Score::INFINITE;
        let ply = Score::MAX_PLY - 1;
        let score = alphabeta(&mut t, &Shared::new(), 3, -window, window, ply);
        assert_eq!(score, Score::mated_in(ply));
        assert!(score.is_loss());
    }

    #[test]
    fn root_move_is_written_to_the_stack() {
        let mut t = state(STALEMATED.replace(" b ", " w ").as_str());
        let _ = search(&mut t, 1);
        assert_ne!(t.stack.at(0).mv, Move::NULL);
        assert_eq!(t.stack.at(1).mv, Move::NULL);
    }

    #[test]
    fn stopped_search_returns_at_once() {
        let mut t = state(BACK_RANK);
        t.can_stop = true;
        t.node_limit = Some(1);
        assert_eq!(search(&mut t, 3), Score::DRAW);
        assert!(t.stopped);
        assert_eq!(t.nodes, 1);
    }

    #[test]
    fn stop_in_a_child_is_not_a_score() {
        let mut t = state(BACK_RANK);
        t.can_stop = true;
        t.node_limit = Some(2);
        assert_eq!(search(&mut t, 3), Score::DRAW);
        assert!(t.stopped);
        assert_eq!(t.nodes, 2);
        assert!(t.pv.line().is_empty());
    }
}
