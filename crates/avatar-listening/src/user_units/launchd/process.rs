use std::io;
use std::path::PathBuf;
use std::time::Duration;

use super::LaunchctlPrint;
use super::effects::LaunchctlControl;
use crate::listener::bounded_command::run_bounded;

pub const LAUNCHCTL: &str = "/bin/launchctl";
pub const COMMAND_TIMEOUT: Duration = Duration::from_secs(30);

pub struct ProcessLaunchctl {
    program: PathBuf,
}

impl ProcessLaunchctl {
    #[must_use]
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
        }
    }

    #[must_use]
    pub fn system() -> Self {
        Self::new(LAUNCHCTL)
    }
}

impl LaunchctlPrint for ProcessLaunchctl {
    fn printed(&mut self, agent_target: &str) -> String {
        run_bounded(&self.program, &["print", agent_target], COMMAND_TIMEOUT)
            .map(|output| output.stdout)
            .unwrap_or_default()
    }
}

impl LaunchctlControl for ProcessLaunchctl {
    fn kickstart(&mut self, target: &str) -> io::Result<()> {
        let output = run_bounded(&self.program, &["kickstart", target], COMMAND_TIMEOUT)?;
        match output.code {
            0 => Ok(()),
            code => Err(io::Error::other(format!(
                "launchctl kickstart exited {code}"
            ))),
        }
    }

    fn terminate(&mut self, target: &str) {
        let _ = run_bounded(&self.program, &["kill", "TERM", target], COMMAND_TIMEOUT);
    }
}
