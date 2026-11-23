pub mod effect_error;
pub mod launchd;
pub mod system_clock;

pub use effect_error::UnitEffectError;
pub use system_clock::SystemClock;

pub const SYSTEMD_UNREACHABLE: i32 = 1;
pub const UNREACHABLE_RETRY_SECONDS: f64 = 30.0;
pub const RETRY_INTERVAL_SECONDS: f64 = 0.5;

pub trait UnitStatus {
    fn exit_code(&mut self, unit: &str) -> i32;
}

pub trait Clock {
    fn monotonic_seconds(&mut self) -> f64;
    fn sleep_seconds(&mut self, seconds: f64);
}

#[must_use]
pub fn unit_active(status: &mut impl UnitStatus, clock: &mut impl Clock, unit: &str) -> bool {
    let deadline = clock.monotonic_seconds() + UNREACHABLE_RETRY_SECONDS;
    let mut code = status.exit_code(unit);
    while code == SYSTEMD_UNREACHABLE && clock.monotonic_seconds() < deadline {
        clock.sleep_seconds(RETRY_INTERVAL_SECONDS);
        code = status.exit_code(unit);
    }
    code == 0
}
