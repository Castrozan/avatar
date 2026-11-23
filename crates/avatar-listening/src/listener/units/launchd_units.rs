use super::SessionUnits;
use crate::user_units::launchd::process::ProcessLaunchctl;
use crate::user_units::launchd::{effects, unit_active};
use crate::user_units::{SystemClock, UnitEffectError};

pub struct LaunchdUnits {
    launchctl: ProcessLaunchctl,
    user_identifier: u32,
}

impl LaunchdUnits {
    #[must_use]
    pub fn new(launchctl: ProcessLaunchctl, user_identifier: u32) -> Self {
        Self {
            launchctl,
            user_identifier,
        }
    }
}

impl SessionUnits for LaunchdUnits {
    fn is_active(&mut self, unit: &str) -> bool {
        unit_active(&mut self.launchctl, self.user_identifier, unit)
    }

    fn start(&mut self, unit: &str) -> Result<(), UnitEffectError> {
        effects::start(&mut self.launchctl, self.user_identifier, unit)
    }

    fn stop(&mut self, unit: &str) -> Result<(), UnitEffectError> {
        let mut clock = SystemClock::new();
        effects::stop(&mut self.launchctl, &mut clock, self.user_identifier, unit)
    }
}
