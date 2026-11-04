use engine::Color;

/// The limits of one search, from `go`. Times are in milliseconds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Limits {
    pub depth: Option<u32>,
    pub nodes: Option<u64>,
    /// `movetime`: the time to spend on this move.
    pub movetime: Option<u64>,
    /// `wtime` and `btime`, indexed by `Color::index()`.
    pub time: [Option<u64>; Color::COUNT],
    /// `winc` and `binc`, indexed by `Color::index()`.
    pub inc: [Option<u64>; Color::COUNT],
    pub movestogo: Option<u32>,
    pub infinite: bool,
    /// Kept for the GUI and the network on each move. Set by the caller from
    /// the `Move Overhead` option.
    pub move_overhead: u64,
}

impl Limits {
    /// A search limited to `depth` only: bench and tests.
    #[must_use]
    pub fn depth(depth: u32) -> Limits {
        Limits {
            depth: Some(depth),
            ..Limits::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_no_limit() {
        let l = Limits::default();
        assert_eq!(l.depth, None);
        assert_eq!(l.nodes, None);
        assert_eq!(l.movetime, None);
        assert_eq!(l.time, [None, None]);
        assert_eq!(l.inc, [None, None]);
        assert_eq!(l.movestogo, None);
        assert!(!l.infinite);
        assert_eq!(l.move_overhead, 0);
    }

    #[test]
    fn depth_sets_only_depth() {
        let l = Limits::depth(5);
        assert_eq!(l.depth, Some(5));
        assert_eq!(Limits { depth: None, ..l }, Limits::default());
    }
}
