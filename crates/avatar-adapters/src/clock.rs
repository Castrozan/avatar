use std::sync::OnceLock;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

static ORIGIN: OnceLock<Instant> = OnceLock::new();

#[must_use]
pub fn wall_seconds() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0.0, |elapsed| elapsed.as_secs_f64())
}

#[must_use]
pub fn monotonic_seconds() -> f64 {
    ORIGIN.get_or_init(Instant::now).elapsed().as_secs_f64()
}
