use std::sync::atomic::{AtomicBool, Ordering};

/// What the search threads share with each other and with the UCI thread.
///
/// Relaxed ordering: the stop flag only has to be seen eventually, nothing
/// else is synchronized through it.
#[derive(Debug, Default)]
pub(crate) struct Shared {
    stop: AtomicBool,
}

impl Shared {
    pub(crate) const fn new() -> Shared {
        Shared {
            stop: AtomicBool::new(false),
        }
    }

    pub(crate) fn request_stop(&self) {
        self.stop.store(true, Ordering::Relaxed);
    }

    pub(crate) fn clear_stop(&self) {
        self.stop.store(false, Ordering::Relaxed);
    }

    pub(crate) fn stop_requested(&self) -> bool {
        self.stop.load(Ordering::Relaxed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stop_is_requested_then_cleared() {
        let s = Shared::new();
        assert!(!s.stop_requested());
        s.request_stop();
        assert!(s.stop_requested());
        s.clear_stop();
        assert!(!s.stop_requested());
    }
}
