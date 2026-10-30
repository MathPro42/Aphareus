use engine::Move;
use eval::Score;

const MAX_PLY: usize = Score::MAX_PLY;

/// Triangular table: the line of a ply is its best move followed by the line
/// of the next ply. Every node clears its own line on entry, or a stale line
/// from an earlier branch is copied up.
#[derive(Debug)]
pub(crate) struct PvTable {
    moves: [[Move; MAX_PLY]; MAX_PLY],
    lens: [usize; MAX_PLY],
}

impl PvTable {
    pub(crate) const fn new() -> PvTable {
        PvTable {
            moves: [[Move::NULL; MAX_PLY]; MAX_PLY],
            lens: [0; MAX_PLY],
        }
    }

    #[inline]
    pub(crate) fn clear_ply(&mut self, ply: usize) {
        self.lens[ply] = 0;
    }

    /// The line of `ply` becomes `mv` followed by the line of `ply + 1`.
    /// Requires `ply < MAX_PLY - 1`.
    #[inline]
    pub(crate) fn update(&mut self, ply: usize, mv: Move) {
        debug_assert!(ply < MAX_PLY - 1);
        let n = self.lens[ply + 1];
        let (head, tail) = self.moves.split_at_mut(ply + 1);
        let line = &mut head[ply];
        line[0] = mv;
        line[1..=n].copy_from_slice(&tail[0][..n]);
        self.lens[ply] = n + 1;
    }

    /// The principal variation from the root.
    pub(crate) fn line(&self) -> &[Move] {
        &self.moves[0][..self.lens[0]]
    }
}

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use engine::Square;

    use super::*;

    const E4: Move = Move::double_push(Square::E2, Square::E4);
    const E5: Move = Move::double_push(Square::E7, Square::E5);
    const NF3: Move = Move::quiet(Square::G1, Square::F3);
    const D4: Move = Move::double_push(Square::D2, Square::D4);

    #[test]
    fn new_line_is_empty() {
        assert!(PvTable::new().line().is_empty());
    }

    #[test]
    fn cascade_builds_the_root_line() {
        let mut pv = Box::new(PvTable::new());
        pv.clear_ply(3);
        pv.update(2, NF3);
        pv.update(1, E5);
        pv.update(0, E4);
        assert_eq!(pv.line(), [E4, E5, NF3]);
    }

    #[test]
    fn cleared_child_drops_its_stale_line() {
        let mut pv = Box::new(PvTable::new());
        pv.clear_ply(3);
        pv.update(2, NF3);
        pv.update(1, E5);
        pv.update(0, E4);
        pv.clear_ply(1);
        pv.update(0, D4);
        assert_eq!(pv.line(), [D4]);
    }

    #[test]
    fn longest_line_fits() {
        let mut pv = Box::new(PvTable::new());
        pv.clear_ply(MAX_PLY - 1);
        for ply in (0..MAX_PLY - 1).rev() {
            pv.update(ply, E4);
        }
        assert_eq!(pv.line().len(), MAX_PLY - 1);
        assert!(pv.line().iter().all(|&m| m == E4));
    }

    #[test]
    fn size_is_the_two_arrays() {
        let expected = MAX_PLY * MAX_PLY * size_of::<Move>() + MAX_PLY * size_of::<usize>();
        assert_eq!(size_of::<PvTable>(), expected);
    }
}
