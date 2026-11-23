use std::thread;
use std::time::{Duration, Instant};

use avatar_holder::floor_holder::{FloorClock, HoldingInterruption};

use super::stop_signal::StopSignal;
use crate::clock::monotonic_seconds;

const SLICE: Duration = Duration::from_millis(20);

pub struct StoppableClock {
    stop: StopSignal,
}

impl StoppableClock {
    #[must_use]
    pub const fn new(stop: StopSignal) -> Self {
        Self { stop }
    }
}

impl FloorClock for StoppableClock {
    fn now_seconds(&mut self) -> f64 {
        monotonic_seconds()
    }

    fn pause(&mut self, seconds: f64) -> Result<(), HoldingInterruption> {
        let deadline = Instant::now() + Duration::from_secs_f64(seconds.max(0.0));
        pause_until(&self.stop, deadline)
    }
}

fn pause_until(stop: &StopSignal, deadline: Instant) -> Result<(), HoldingInterruption> {
    let slices = std::iter::from_fn(|| (!stop.is_raised()).then(|| sleep_slice(deadline)));
    let reached = slices.into_iter().any(|more| !more);
    reached.then_some(()).ok_or(HoldingInterruption::Stopped)
}

fn sleep_slice(deadline: Instant) -> bool {
    let remaining = deadline.saturating_duration_since(Instant::now());
    thread::sleep(remaining.min(SLICE));
    !remaining.is_zero()
}
