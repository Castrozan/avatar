mod launchd_units;
mod systemd_units;

use crate::user_units::UnitEffectError;

pub use launchd_units::LaunchdUnits;
pub use systemd_units::SystemdUnits;

pub trait SessionUnits {
    fn is_active(&mut self, unit: &str) -> bool;
    fn start(&mut self, unit: &str) -> Result<(), UnitEffectError>;
    fn stop(&mut self, unit: &str) -> Result<(), UnitEffectError>;
}

impl<Units: SessionUnits + ?Sized> SessionUnits for &mut Units {
    fn is_active(&mut self, unit: &str) -> bool {
        (**self).is_active(unit)
    }

    fn start(&mut self, unit: &str) -> Result<(), UnitEffectError> {
        (**self).start(unit)
    }

    fn stop(&mut self, unit: &str) -> Result<(), UnitEffectError> {
        (**self).stop(unit)
    }
}
