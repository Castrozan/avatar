use std::fs::{self, File, OpenOptions};
use std::io;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use avatar_conversation::voice_turns::{SpeechRequest, speech_command};
use avatar_holder::floor_holder::{Speech, SpeechFailure};

use crate::commands::Programs;
use crate::process::{Captured, run_captured};

const SPEECH_TIMEOUT: Duration = Duration::from_secs(310);
const SPOKEN_FILE: &str = "spoken";

pub struct SeatSpeech<Output: FnMut() -> String> {
    directory: PathBuf,
    programs: Programs,
    speech_output: Output,
}

impl<Output: FnMut() -> String> SeatSpeech<Output> {
    pub fn new(directory: impl Into<PathBuf>, programs: Programs, speech_output: Output) -> Self {
        Self {
            directory: directory.into(),
            programs,
            speech_output,
        }
    }

    fn mark_spoken(&self) -> io::Result<()> {
        fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(&self.directory)?;
        let mark = open_mark(&self.directory.join(SPOKEN_FILE))?;
        mark.set_modified(SystemTime::now())
    }
}

fn open_mark(path: &std::path::Path) -> io::Result<File> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .open(path)
}

impl<Output: FnMut() -> String> Speech for SeatSpeech<Output> {
    fn speak(&mut self, text: &str) -> Result<(), SpeechFailure> {
        self.mark_spoken().map_err(|error| failure(&error))?;
        let command = speech_command(&neutral(text), &(self.speech_output)());
        let command = self.programs.resolve(command);
        let captured = run_captured(&command, &[], SPEECH_TIMEOUT);
        exit_of(&captured.map_err(|error| failure(&error))?)
    }
}

fn neutral(text: &str) -> SpeechRequest {
    SpeechRequest {
        text: text.to_owned(),
        emotion: "neutral".to_owned(),
        voice: None,
    }
}

fn exit_of(captured: &Captured) -> Result<(), SpeechFailure> {
    let message = format!("speech exited {}", captured.exit_code);
    captured
        .succeeded()
        .then_some(())
        .ok_or_else(|| SpeechFailure::new(message))
}

fn failure(error: &io::Error) -> SpeechFailure {
    SpeechFailure::new(error.to_string())
}
