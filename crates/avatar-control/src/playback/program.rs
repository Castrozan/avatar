use std::path::Path;

use super::command::ProgramCommand;
use super::{AudioPlayer, Playback};

pub struct ProgramAudioPlayer {
    command: ProgramCommand,
}

impl ProgramAudioPlayer {
    pub fn new(program: impl Into<String>, arguments: Vec<String>) -> Self {
        Self {
            command: ProgramCommand::new(program.into(), arguments, Vec::new()),
        }
    }
}

impl AudioPlayer for ProgramAudioPlayer {
    fn play(&self, audio_path: &Path) -> Playback {
        self.command.play(audio_path)
    }
}
