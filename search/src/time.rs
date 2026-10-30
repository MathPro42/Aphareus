use std::time::Instant;

use engine::Color;

use crate::Limits;

/// Moves to go when `movestogo` is absent, and its upper bound.
const MOVES_TO_GO: u64 = 30;

/// When to stop. The soft limit is checked between iterations, the hard one
/// during the search. Both in milliseconds since `start`.
#[derive(Debug)]
pub(crate) struct TimeManager {
    start: Instant,
    soft: Option<u64>,
    hard: Option<u64>,
}

impl TimeManager {
    /// Starts the clock now, for the side to move `us`.
    pub(crate) fn new(limits: &Limits, us: Color) -> TimeManager {
        let (soft, hard) = budget(limits, us);
        TimeManager {
            start: Instant::now(),
            soft,
            hard,
        }
    }

    /// Milliseconds since the search started.
    pub(crate) fn elapsed_ms(&self) -> u64 {
        u64::try_from(self.start.elapsed().as_millis()).unwrap_or(u64::MAX)
    }

    /// Returns `true` if no new iteration should start.
    pub(crate) fn soft_exceeded(&self) -> bool {
        self.soft.is_some_and(|soft| self.elapsed_ms() >= soft)
    }

    /// Returns `true` if the search must stop now.
    pub(crate) fn hard_exceeded(&self) -> bool {
        self.hard.is_some_and(|hard| self.elapsed_ms() >= hard)
    }
}

/// The soft and hard limits in ms, `None` without a time limit. About 1/30 of
/// the time left plus 3/4 of the increment, never more than 3/4 of the time
/// left, never zero. Saturating: absurd GUI values must not overflow.
fn budget(limits: &Limits, us: Color) -> (Option<u64>, Option<u64>) {
    if limits.infinite {
        return (None, None);
    }
    if let Some(movetime) = limits.movetime {
        let t = movetime.saturating_sub(limits.move_overhead).max(1);
        return (Some(t), Some(t));
    }
    let Some(time) = limits.time[us.index()] else {
        return (None, None);
    };
    let left = time.saturating_sub(limits.move_overhead).max(1);
    let inc = limits.inc[us.index()].unwrap_or(0);
    let mtg = limits
        .movestogo
        .map_or(MOVES_TO_GO, u64::from)
        .clamp(1, MOVES_TO_GO);
    let soft = (left / mtg).saturating_add(three_quarters(inc));
    let hard = soft.saturating_mul(4).min(three_quarters(left));
    let soft = soft.min(hard);
    (Some(soft.max(1)), Some(hard.max(1)))
}

/// `x × 3 / 4`, rounded down, without overflow.
const fn three_quarters(x: u64) -> u64 {
    x / 4 * 3 + x % 4 * 3 / 4
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;

    use super::*;

    const WHITE: usize = Color::White.index();
    const BLACK: usize = Color::Black.index();

    fn clock(us: usize, time: u64, inc: Option<u64>, overhead: u64) -> Limits {
        let mut l = Limits {
            move_overhead: overhead,
            ..Limits::default()
        };
        l.time[us] = Some(time);
        l.inc[us] = inc;
        l
    }

    #[test]
    fn about_a_thirtieth_of_the_time() {
        let l = clock(WHITE, 60_000, None, 10);
        assert_eq!(budget(&l, Color::White), (Some(1999), Some(7996)));
    }

    #[test]
    fn increment_adds_three_quarters() {
        let l = clock(WHITE, 60_000, Some(1000), 10);
        assert_eq!(budget(&l, Color::White), (Some(2749), Some(10_996)));
    }

    #[test]
    fn black_reads_its_own_clock() {
        let mut l = clock(BLACK, 60_000, Some(1000), 10);
        l.time[WHITE] = Some(1000);
        l.inc[WHITE] = Some(0);
        assert_eq!(budget(&l, Color::Black), (Some(2749), Some(10_996)));
        assert_eq!(budget(&l, Color::White), (Some(33), Some(132)));
    }

    #[test]
    fn movetime_is_both_limits() {
        let l = Limits {
            movetime: Some(100),
            move_overhead: 10,
            ..Limits::default()
        };
        assert_eq!(budget(&l, Color::White), (Some(90), Some(90)));
    }

    #[test]
    fn short_clock_is_never_zero() {
        let l = clock(WHITE, 100, None, 0);
        assert_eq!(budget(&l, Color::White), (Some(3), Some(12)));
    }

    #[test]
    fn last_move_before_control_keeps_a_quarter() {
        let mut l = clock(WHITE, 1000, None, 10);
        l.movestogo = Some(1);
        assert_eq!(budget(&l, Color::White), (Some(742), Some(742)));
    }

    #[test]
    fn movestogo_is_clamped() {
        let mut l = clock(WHITE, 1000, None, 0);
        l.movestogo = Some(0);
        assert_eq!(budget(&l, Color::White), (Some(750), Some(750)));
        l.movestogo = Some(u32::MAX);
        assert_eq!(budget(&l, Color::White), (Some(33), Some(132)));
    }

    #[test]
    fn overhead_above_the_clock_gives_one_ms() {
        let l = clock(WHITE, 5, Some(0), 10);
        assert_eq!(budget(&l, Color::White), (Some(1), Some(1)));
        let l = Limits {
            movetime: Some(5),
            move_overhead: 10,
            ..Limits::default()
        };
        assert_eq!(budget(&l, Color::White), (Some(1), Some(1)));
    }

    #[test]
    fn no_time_limit() {
        assert_eq!(budget(&Limits::default(), Color::White), (None, None));
        assert_eq!(budget(&Limits::depth(5), Color::White), (None, None));
        let nodes = Limits {
            nodes: Some(1000),
            ..Limits::default()
        };
        assert_eq!(budget(&nodes, Color::White), (None, None));
        // The other side's clock does not limit us.
        assert_eq!(
            budget(&clock(BLACK, 1000, None, 0), Color::White),
            (None, None)
        );
    }

    #[test]
    fn infinite_wins_over_any_limit() {
        let mut l = clock(WHITE, 1000, Some(10), 0);
        l.movetime = Some(100);
        l.infinite = true;
        assert_eq!(budget(&l, Color::White), (None, None));
    }

    #[test]
    fn huge_values_do_not_overflow() {
        let l = clock(WHITE, u64::MAX, Some(u64::MAX), 0);
        let max = three_quarters(u64::MAX);
        assert_eq!(budget(&l, Color::White), (Some(max), Some(max)));
        let l = Limits {
            movetime: Some(u64::MAX),
            ..Limits::default()
        };
        assert_eq!(budget(&l, Color::White), (Some(u64::MAX), Some(u64::MAX)));
    }

    #[test]
    fn three_quarters_rounds_down() {
        for x in [0, 1, 2, 3, 4, 990, 1000, u64::MAX - 1, u64::MAX] {
            let wide = u128::from(x) * 3 / 4;
            assert_eq!(u128::from(three_quarters(x)), wide, "{x}");
        }
    }

    #[test]
    fn new_uses_the_budget() {
        let tm = TimeManager::new(&clock(WHITE, 60_000, None, 10), Color::White);
        assert_eq!((tm.soft, tm.hard), (Some(1999), Some(7996)));
        let tm = TimeManager::new(&Limits::depth(3), Color::White);
        assert_eq!((tm.soft, tm.hard), (None, None));
    }

    #[test]
    fn no_limit_is_never_exceeded() {
        let tm = TimeManager::new(&Limits::default(), Color::White);
        assert!(!tm.soft_exceeded());
        assert!(!tm.hard_exceeded());
    }

    #[test]
    fn limits_are_exceeded_once_reached() {
        let tm = TimeManager {
            start: Instant::now(),
            soft: Some(1),
            hard: Some(u64::MAX),
        };
        assert!(!tm.hard_exceeded());
        thread::sleep(Duration::from_millis(2));
        assert!(tm.elapsed_ms() >= 2);
        assert!(tm.soft_exceeded());
        assert!(!tm.hard_exceeded());
        let tm = TimeManager {
            hard: Some(1),
            ..tm
        };
        assert!(tm.hard_exceeded());
    }
}
