use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use avatar_control::speaking::runtime::Clock;

#[derive(Clone)]
pub(crate) struct FakeClock(pub(crate) Arc<AtomicU64>);

impl FakeClock {
    pub(crate) fn starting_at(now_ms: u64) -> Self {
        Self(Arc::new(AtomicU64::new(now_ms)))
    }
}

impl Clock for FakeClock {
    fn now_ms(&self) -> u64 {
        self.0.load(Ordering::SeqCst)
    }
}
