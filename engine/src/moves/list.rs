use std::fmt;

use crate::moves::Move;

pub trait MoveSink {
    /// Adds a move.
    fn push(&mut self, m: Move);
}

/// A fixed-capacity list of moves, stored inline.
#[derive(Clone)]
pub struct MoveList {
    moves: [Move; MoveList::CAPACITY],
    len: usize,
}

impl MoveList {
    /// Maximum number of moves.
    pub const CAPACITY: usize = 256;

    /// An empty list.
    #[inline]
    pub const fn new() -> MoveList {
        MoveList {
            moves: [Move::NULL; MoveList::CAPACITY],
            len: 0,
        }
    }

    /// Adds a move at the end.
    #[inline]
    pub const fn push(&mut self, m: Move) {
        debug_assert!(self.len < MoveList::CAPACITY, "move list is full");
        self.moves[self.len] = m;
        self.len += 1;
    }

    /// Number of moves.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if there is no move.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Removes all moves.
    #[inline]
    pub const fn clear(&mut self) {
        self.len = 0;
    }

    /// The moves, in insertion order.
    #[inline]
    pub fn as_slice(&self) -> &[Move] {
        &self.moves[..self.len]
    }

    /// The moves, mutably (for sorting).
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [Move] {
        &mut self.moves[..self.len]
    }

    /// Returns `true` if the Move `m` is in the list.
    #[inline]
    #[must_use]
    pub fn contains(&self, m: Move) -> bool {
        self.as_slice().contains(&m)
    }
}

impl Default for MoveList {
    fn default() -> MoveList {
        MoveList::new()
    }
}

impl MoveSink for MoveList {
    #[inline]
    fn push(&mut self, m: Move) {
        MoveList::push(self, m);
    }
}

impl<'a> IntoIterator for &'a MoveList {
    type Item = &'a Move;
    type IntoIter = std::slice::Iter<'a, Move>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

/// Shows only the moves in the list, not the unused capacity.
impl fmt::Debug for MoveList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.as_slice()).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Square;

    fn sample(i: usize) -> Move {
        Move::quiet(Square::ALL[i % 64], Square::ALL[(i / 64 + 1) % 64])
    }

    #[test]
    fn push_keeps_order() {
        let mut list = MoveList::new();
        let moves = [
            Move::quiet(Square::G1, Square::F3),
            Move::double_push(Square::E2, Square::E4),
            Move::capture(Square::E4, Square::D5),
        ];
        for m in moves {
            list.push(m);
        }
        assert_eq!(list.len(), 3);
        assert_eq!(list.as_slice(), moves);
        assert_eq!((&list).into_iter().copied().collect::<Vec<_>>(), moves);
    }

    #[test]
    fn fill_to_capacity() {
        let mut list = MoveList::new();
        for i in 0..MoveList::CAPACITY {
            list.push(sample(i));
        }
        assert_eq!(list.len(), MoveList::CAPACITY);
        for (i, &m) in list.as_slice().iter().enumerate() {
            assert_eq!(m, sample(i));
        }
    }

    #[test]
    fn clear() {
        let mut list = MoveList::new();
        list.push(sample(0));
        list.push(sample(1));
        list.clear();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
        assert!(list.as_slice().is_empty());
    }

    #[test]
    fn contains() {
        let mut list = MoveList::new();
        let present = Move::quiet(Square::G1, Square::F3);
        let absent = Move::quiet(Square::B1, Square::C3);
        list.push(present);
        assert!(list.contains(present));
        assert!(!list.contains(absent));
        // Same squares, different flag: a different move.
        assert!(!list.contains(Move::capture(Square::G1, Square::F3)));
    }

    #[test]
    fn works_as_sink() {
        fn generate<S: MoveSink>(sink: &mut S) {
            sink.push(Move::double_push(Square::E2, Square::E4));
        }
        let mut list = MoveList::new();
        generate(&mut list);
        assert_eq!(list.as_slice(), [Move::double_push(Square::E2, Square::E4)]);
    }
}
