use std::time::{Duration, Instant};

use avatar_live::boundaries::Clock;
use tokio::time::sleep;

#[derive(Clone, Copy)]
pub(crate) struct RunClock {
    started: Instant,
}

impl RunClock {
    pub(crate) fn new() -> Self {
        Self {
            started: Instant::now(),
        }
    }
}

impl Clock for RunClock {
    fn now(&self) -> f64 {
        self.started.elapsed().as_secs_f64()
    }

    async fn sleep(&mut self, seconds: f64) {
        sleep(Duration::from_secs_f64(seconds)).await;
    }
}
