use avatar_listening::listener::units::SessionUnits;
use avatar_listening::user_units::UnitEffectError;

pub(crate) struct FakeUnits {
    pub(crate) active: Vec<String>,
    pub(crate) events: Vec<Vec<String>>,
}

impl FakeUnits {
    pub(crate) fn with_active(active: Vec<String>) -> Self {
        Self {
            active,
            events: Vec::new(),
        }
    }
}

impl SessionUnits for FakeUnits {
    fn is_active(&mut self, unit: &str) -> bool {
        self.active.iter().any(|name| name == unit)
    }

    fn start(&mut self, unit: &str) -> Result<(), UnitEffectError> {
        self.events.push(vec!["start".into(), unit.into()]);
        Ok(())
    }

    fn stop(&mut self, unit: &str) -> Result<(), UnitEffectError> {
        self.events.push(vec!["stop".into(), unit.into()]);
        Ok(())
    }
}
