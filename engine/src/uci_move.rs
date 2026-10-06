use crate::board::{Board, CastlingConfig};
use crate::movegen::{All, Context, generate};
use crate::moves::{Move, MoveList};

/// `m`, a move of `board`, in UCI notation. [`Move::NULL`] is `0000`.
#[must_use]
pub fn to_uci(board: &Board, m: Move, chess960: bool) -> String {
    if m.is_null() {
        return "0000".to_string();
    }

    let to = if m.is_castle() && !chess960 {
        CastlingConfig::king_destination(board.side_to_move(), m.castle_side())
    } else {
        m.to()
    };
    let mut text = format!("{}{to}", m.from());
    if let Some(piece) = m.promotion_piece() {
        text.push(piece.to_char());
    }
    text
}

/// The legal move of `board` written `text`, or `None` if there is none.
#[must_use]
pub fn from_uci(board: &Board, text: &str, chess960: bool) -> Option<Move> {
    let mut moves = MoveList::new();
    generate::<All>(&Context::new(board), &mut moves);
    moves.as_slice().iter().copied().find(|&m| {
        to_uci(board, m, chess960) == text
            || (m.is_castle() && !chess960 && to_uci(board, m, true) == text)
    })
}
