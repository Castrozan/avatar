use std::thread;
use std::time::Duration;

use avatar_adapters::clock::{monotonic_seconds, wall_seconds};
use avatar_conversation::talk::ports::Clock;

pub(crate) struct SystemClock;

impl Clock for SystemClock {
    fn monotonic(&self) -> f64 {
        monotonic_seconds()
    }

    fn wall(&self) -> f64 {
        wall_seconds()
    }

    fn sleep(&mut self, seconds: f64) {
        thread::sleep(Duration::try_from_secs_f64(seconds).unwrap_or_default());
    }
}
