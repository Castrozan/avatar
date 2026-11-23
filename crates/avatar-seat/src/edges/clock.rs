use std::time::{Duration, Instant};

pub trait Clock {
    fn monotonic(&self) -> f64;
    fn sleep(&self, seconds: f64);
}

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
    fn monotonic(&self) -> f64 {
        self.origin.elapsed().as_secs_f64()
    }

    fn sleep(&self, seconds: f64) {
        std::thread::sleep(Duration::from_secs_f64(seconds));
    }
}
