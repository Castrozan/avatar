use std::path::Path;
use std::process::Stdio;

use tokio::process::Command;

use super::handle::Playback;

#[derive(Debug, Clone)]
pub(super) struct ProgramCommand {
    program: String,
    arguments: Vec<String>,
    environment: Vec<(String, String)>,
}

impl ProgramCommand {
    pub(super) fn new(
        program: String,
        arguments: Vec<String>,
        environment: Vec<(String, String)>,
    ) -> Self {
        Self {
            program,
            arguments,
            environment,
        }
    }

    pub(super) fn play(&self, audio_path: &Path) -> Playback {
        match self.command(audio_path).spawn() {
            Ok(child) => Playback::supervise(child),
            Err(_) => Playback::unstarted(),
        }
    }

    fn command(&self, audio_path: &Path) -> Command {
        let mut command = Command::new(&self.program);
        command
            .args(&self.arguments)
            .arg(audio_path)
            .envs(self.environment.iter().map(|(key, value)| (key, value)))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());
        command
    }
}
