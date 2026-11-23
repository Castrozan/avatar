use std::path::{Path, PathBuf};
use std::process::Command;

use super::control::{PulseControl, PulseError};

pub struct PactlAdapter {
    program: PathBuf,
}

impl PactlAdapter {
    #[must_use]
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
        }
    }
}

impl PulseControl for PactlAdapter {
    fn run(&mut self, arguments: &[&str]) -> Result<String, PulseError> {
        let output = Command::new(&self.program)
            .args(arguments)
            .output()
            .map_err(|error| spawn_error(&self.program, &error))?;
        stdout_of_successful(&output)
    }
}

fn spawn_error(program: &Path, error: &std::io::Error) -> PulseError {
    PulseError::new(format!("cannot start {}: {error}", program.display()))
}

fn stdout_of_successful(output: &std::process::Output) -> Result<String, PulseError> {
    if !output.status.success() {
        return Err(PulseError::new(format!(
            "pactl exited with {}",
            output.status
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}
