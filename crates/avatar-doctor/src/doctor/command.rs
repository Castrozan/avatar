use super::failure::DoctorError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandOutput {
    pub succeeded: bool,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl CommandOutput {
    #[must_use]
    pub fn last_stdout_line(&self) -> String {
        last_line(&self.stdout)
    }

    #[must_use]
    pub fn last_stderr_line(&self) -> String {
        last_line(&self.stderr)
    }
}

fn last_line(output: &[u8]) -> String {
    String::from_utf8_lossy(output)
        .trim()
        .lines()
        .last()
        .unwrap_or_default()
        .to_owned()
}

pub trait CommandRunner {
    fn run(&self, command: &[&str], stdin: &[u8]) -> Result<CommandOutput, DoctorError>;
}
