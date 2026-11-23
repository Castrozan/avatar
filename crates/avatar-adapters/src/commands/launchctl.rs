use std::time::Duration;

use super::programs::Programs;
use crate::process::run_captured;

const PROBE_TIMEOUT: Duration = Duration::from_secs(15);

pub struct Launchctl {
    programs: Programs,
}

impl Launchctl {
    #[must_use]
    pub const fn new(programs: Programs) -> Self {
        Self { programs }
    }

    #[must_use]
    pub fn agent_loaded(&self, user_id: u32, label: &str) -> bool {
        let target = format!("gui/{user_id}/{label}");
        let command = self.programs.words(&["launchctl", "print", &target]);
        run_captured(&command, &[], PROBE_TIMEOUT).is_ok_and(|captured| captured.succeeded())
    }
}
