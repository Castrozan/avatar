use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub trait Timing {
    fn wall_seconds(&mut self) -> f64;
    fn pause(&mut self, seconds: f64);
}

#[must_use]
pub fn wall_seconds_now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0.0, |elapsed| elapsed.as_secs_f64())
}

pub struct SystemTiming;

impl Timing for SystemTiming {
    fn wall_seconds(&mut self) -> f64 {
        wall_seconds_now()
    }

    fn pause(&mut self, seconds: f64) {
        thread::sleep(Duration::from_secs_f64(seconds));
    }
}
