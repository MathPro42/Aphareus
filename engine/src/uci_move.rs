use crate::board::{Board, CastlingConfig};
use crate::movegen::legal_moves;
use crate::moves::Move;
use crate::types::{PieceType, Square};

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
    let (from, to, promotion) = parse(text)?;
    let us = board.side_to_move();
    legal_moves(board).as_slice().iter().copied().find(|&m| {
        m.from() == from
            && m.promotion_piece() == promotion
            && (m.to() == to
                || (m.is_castle()
                    && !chess960
                    && CastlingConfig::king_destination(us, m.castle_side()) == to))
    })
}

/// The squares and promotion piece of `text`, such as `e7e8q`, without
/// checking that they make a move.
fn parse(text: &str) -> Option<(Square, Square, Option<PieceType>)> {
    if !(4..=5).contains(&text.len()) || !text.is_ascii() {
        return None;
    }
    let from = Square::parse(&text[0..2])?;
    let to = Square::parse(&text[2..4])?;
    let promotion = match text.as_bytes().get(4) {
        Some(&letter) => Some(PieceType::from_char(char::from(letter))?),
        None => None,
    };
    Some((from, to, promotion))
}
