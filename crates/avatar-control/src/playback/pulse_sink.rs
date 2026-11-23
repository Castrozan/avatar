use std::path::Path;

use super::command::ProgramCommand;
use super::{AudioPlayer, Playback};

const PLAYBACK_CLIENT_NAME: &str = "avatar-speech";

pub struct PulseSinkPlayer {
    command: ProgramCommand,
}

impl PulseSinkPlayer {
    pub fn new(
        program: impl Into<String>,
        sink_name: impl Into<String>,
        runtime_directory: impl Into<String>,
    ) -> Self {
        let arguments = pulse_arguments(sink_name.into());
        let environment = vec![("XDG_RUNTIME_DIR".to_owned(), runtime_directory.into())];
        Self {
            command: ProgramCommand::new(program.into(), arguments, environment),
        }
    }
}

fn pulse_arguments(sink_name: String) -> Vec<String> {
    vec![
        "--client-name".to_owned(),
        PLAYBACK_CLIENT_NAME.to_owned(),
        "--device".to_owned(),
        sink_name,
    ]
}

impl AudioPlayer for PulseSinkPlayer {
    fn play(&self, audio_path: &Path) -> Playback {
        self.command.play(audio_path)
    }
}
