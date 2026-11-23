use std::thread;
use std::time::{Duration, Instant};

use super::Clock;

pub struct SystemClock {
    origin: Instant,
}

impl SystemClock {
    #[must_use]
    pub fn new() -> Self {
        Self {
            origin: Instant::now(),
        }
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for SystemClock {
    fn monotonic_seconds(&mut self) -> f64 {
        self.origin.elapsed().as_secs_f64()
    }

    fn sleep_seconds(&mut self, seconds: f64) {
        thread::sleep(Duration::from_secs_f64(seconds));
    }
}
