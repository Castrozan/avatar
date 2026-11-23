use avatar_listening::listener::units::{LaunchdUnits, SessionUnits};
use avatar_listening::user_units::launchd::process::ProcessLaunchctl;

use crate::edge::failure::unit_failure;
use crate::failure::Failure;
use crate::stage::StageUnits;

pub struct LaunchdStageUnits {
    units: LaunchdUnits,
}

impl LaunchdStageUnits {
    #[must_use]
    pub fn new(launchctl: ProcessLaunchctl, user_identifier: u32) -> Self {
        Self {
            units: LaunchdUnits::new(launchctl, user_identifier),
        }
    }
}

impl StageUnits for LaunchdStageUnits {
    fn unit_active(&mut self, unit: &str) -> bool {
        self.units.is_active(unit)
    }

    fn start_unit(&mut self, unit: &str) -> Result<(), Failure> {
        self.units.start(unit).map_err(|error| unit_failure(&error))
    }

    fn stop_unit(&mut self, unit: &str) -> Result<(), Failure> {
        self.units.stop(unit).map_err(|error| unit_failure(&error))
    }
}
