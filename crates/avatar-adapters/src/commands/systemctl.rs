use std::time::Duration;

use super::programs::Programs;
use crate::process::run_captured;

const PROBE_TIMEOUT: Duration = Duration::from_secs(15);

pub struct Systemctl {
    programs: Programs,
}

impl Systemctl {
    #[must_use]
    pub const fn new(programs: Programs) -> Self {
        Self { programs }
    }

    #[must_use]
    pub fn unit_state(&self, unit: &str) -> Option<String> {
        let command = self
            .programs
            .words(&["systemctl", "--user", "is-active", unit]);
        let captured = run_captured(&command, &[], PROBE_TIMEOUT).ok()?;
        let text = String::from_utf8_lossy(&captured.stdout);
        text.trim().lines().last().map(str::to_owned)
    }

    #[must_use]
    pub fn unit_active(&self, unit: &str) -> bool {
        self.unit_state(unit).as_deref() == Some("active")
    }
}
