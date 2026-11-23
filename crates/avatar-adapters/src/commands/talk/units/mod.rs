use avatar_conversation::talk::TalkFailure;
use avatar_conversation::talk::ports::exec::run_unchecked;
use avatar_conversation::talk::ports::{Readiness, Units};
use avatar_listening::user_units::{Clock, UnitStatus, unit_active};

use bring_up::bring_up;
use waiting::wait_for_units;

use super::health::HealthProbe;
use super::process::TalkProcess;
use crate::clock::monotonic_seconds;

mod bring_up;
mod readiness;
mod waiting;

#[derive(Clone, Copy, Debug)]
pub struct ReadinessTiming {
    pub timeout_seconds: f64,
    pub poll_seconds: f64,
}

impl Default for ReadinessTiming {
    fn default() -> Self {
        Self {
            timeout_seconds: 45.0,
            poll_seconds: 0.25,
        }
    }
}

pub struct SystemdUnits {
    process: TalkProcess,
    health: HealthProbe,
    timing: ReadinessTiming,
}

impl SystemdUnits {
    #[must_use]
    pub const fn new(process: TalkProcess, health: HealthProbe, timing: ReadinessTiming) -> Self {
        Self {
            process,
            health,
            timing,
        }
    }
}

struct SystemctlStatus<'process>(&'process mut TalkProcess);

impl UnitStatus for SystemctlStatus<'_> {
    fn exit_code(&mut self, unit: &str) -> i32 {
        let argv = ["systemctl", "--user", "is-active", "--quiet", unit];
        run_unchecked(self.0, &argv, None).map_or(-1, |output| output.status)
    }
}

struct SleepingClock;

impl Clock for SleepingClock {
    fn monotonic_seconds(&mut self) -> f64 {
        monotonic_seconds()
    }

    fn sleep_seconds(&mut self, seconds: f64) {
        std::thread::sleep(std::time::Duration::from_secs_f64(seconds));
    }
}

impl Units for SystemdUnits {
    fn unit_active(&mut self, unit: &str) -> bool {
        let mut status = SystemctlStatus(&mut self.process);
        unit_active(&mut status, &mut SleepingClock, unit)
    }

    fn bring_up(&mut self, target: &str, checks: &[Readiness]) -> Result<(), TalkFailure> {
        bring_up(self, target, checks)
    }

    fn wait_until_ready(
        &mut self,
        units: &[&str],
        checks: &[Readiness],
    ) -> Result<(), TalkFailure> {
        wait_for_units(self, units, checks)
    }

    fn remote_seat_listening(&mut self) -> bool {
        self.health.remote_seat_listening()
    }
}
