use std::thread;
use std::time::{Duration, Instant};

use crate::waiting::Clock;

#[derive(Debug, Clone, Copy)]
pub struct SystemWaiting {
    origin: Instant,
}

impl SystemWaiting {
    #[must_use]
    pub fn new() -> Self {
        Self {
            origin: Instant::now(),
        }
    }
}

impl Default for SystemWaiting {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for SystemWaiting {
    fn monotonic(&self) -> f64 {
        self.origin.elapsed().as_secs_f64()
    }

    fn sleep(&mut self, seconds: f64) {
        thread::sleep(Duration::from_secs_f64(seconds.max(0.0)));
    }
}
