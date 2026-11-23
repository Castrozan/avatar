use std::path::PathBuf;
use std::time::Duration;

use super::SessionUnits;
use crate::listener::bounded_command::run_bounded;
use crate::user_units::{SystemClock, UnitEffectError, UnitStatus, unit_active};

const COMMAND_TIMEOUT: Duration = Duration::from_mins(2);

pub struct SystemdUnits {
    program: PathBuf,
}

impl SystemdUnits {
    #[must_use]
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
        }
    }

    fn systemctl(&self, verb: &str, unit: &str) -> Result<(), UnitEffectError> {
        let output = run_bounded(&self.program, &["--user", verb, unit], COMMAND_TIMEOUT)?;
        match output.code {
            0 => Ok(()),
            code => Err(std::io::Error::other(format!("systemctl {verb} exited {code}")).into()),
        }
    }
}

impl UnitStatus for SystemdUnits {
    fn exit_code(&mut self, unit: &str) -> i32 {
        let arguments = ["--user", "is-active", "--quiet", unit];
        run_bounded(&self.program, &arguments, COMMAND_TIMEOUT).map_or(-1, |output| output.code)
    }
}

impl SessionUnits for SystemdUnits {
    fn is_active(&mut self, unit: &str) -> bool {
        unit_active(self, &mut SystemClock::new(), unit)
    }

    fn start(&mut self, unit: &str) -> Result<(), UnitEffectError> {
        self.systemctl("start", unit)
    }

    fn stop(&mut self, unit: &str) -> Result<(), UnitEffectError> {
        self.systemctl("stop", unit)
    }
}
