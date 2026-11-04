use engine::Move;
use eval::Score;

/// What a node and its parent or children pass to each other.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct StackEntry {
    /// The move played at this ply, `NULL` at the root.
    pub(crate) mv: Move,
    /// Static evaluation of this node, `NONE` if not computed.
    pub(crate) eval: Score,
}

impl StackEntry {
    const EMPTY: StackEntry = StackEntry {
        mv: Move::NULL,
        eval: Score::NONE,
    };
}

/// Spare entries before the root, for a future `back(ply, k)` that reads
/// `k <= MARGIN` plies back near the root. `at(ply - k)` would underflow.
const MARGIN: usize = 8;

/// One entry per ply. Entry `ply` is stored at `ply + MARGIN`.
#[derive(Debug)]
pub(crate) struct Stack {
    entries: [StackEntry; Score::MAX_PLY + MARGIN],
}

impl Stack {
    pub(crate) const fn new() -> Stack {
        Stack {
            entries: [StackEntry::EMPTY; Score::MAX_PLY + MARGIN],
        }
    }

    #[inline]
    pub(crate) fn at(&mut self, ply: usize) -> &mut StackEntry {
        &mut self.entries[ply + MARGIN]
    }

    /// Resets every entry, the margin included.
    pub(crate) fn clear(&mut self) {
        self.entries.fill(StackEntry::EMPTY);
    }
}

#[cfg(test)]
mod tests {
    use engine::Square;

    use super::*;

    const E4: Move = Move::double_push(Square::E2, Square::E4);

    fn all_empty(s: &Stack) -> bool {
        s.entries.iter().all(|e| *e == StackEntry::EMPTY)
    }

    #[test]
    fn new_stack_is_empty() {
        assert!(all_empty(&Stack::new()));
    }

    #[test]
    fn write_stays_at_its_ply() {
        let mut s = Stack::new();
        *s.at(3) = StackEntry {
            mv: E4,
            eval: Score::new(25),
        };
        assert_eq!(s.at(3).mv, E4);
        assert_eq!(s.at(3).eval, Score::new(25));
        assert_eq!(*s.at(2), StackEntry::EMPTY);
        assert_eq!(*s.at(4), StackEntry::EMPTY);
    }

    #[test]
    fn root_and_deepest_ply_are_valid() {
        let mut s = Stack::new();
        s.at(0).mv = E4;
        s.at(Score::MAX_PLY - 1).eval = Score::DRAW;
        assert_eq!(s.at(0).mv, E4);
        assert_eq!(s.at(Score::MAX_PLY - 1).eval, Score::DRAW);
    }

    #[test]
    fn clear_resets_everything() {
        let mut s = Stack::new();
        s.at(0).mv = E4;
        s.at(Score::MAX_PLY - 1).eval = Score::DRAW;
        s.entries[0].eval = Score::DRAW;
        s.clear();
        assert!(all_empty(&s));
    }
}
