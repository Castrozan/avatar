mod output;
mod process;

pub mod elevenlabs;
pub mod emotion;
pub mod kokoro;

use std::path::{Path, PathBuf};
use std::process::{Output, Stdio};
use std::time::Duration;

use tokio::process::{Child, Command};

pub use output::{MediaAlignment, MediaSpeechOutput};

use super::contract::SynthesisError;
use crate::speech::spoken_language::SpokenLanguage;

#[must_use]
pub fn language_tag(language: SpokenLanguage) -> &'static str {
    match language {
        SpokenLanguage::PortugueseBrazil => "pt-br",
        SpokenLanguage::EnglishUs => "en-us",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaSpeechClient {
    program: PathBuf,
}

impl MediaSpeechClient {
    #[must_use]
    pub fn new(program: PathBuf) -> Self {
        Self { program }
    }

    pub async fn run(
        &self,
        arguments: &[&str],
        timeout: Duration,
    ) -> Result<MediaSpeechOutput, SynthesisError> {
        let child = spawn(&self.program, arguments)?;
        let output = process::wait_with_timeout(child, timeout).await?;
        MediaSpeechOutput::parse(&successful_stdout(output)?)
    }
}

fn successful_stdout(output: Output) -> Result<Vec<u8>, SynthesisError> {
    if output.status.success() {
        return Ok(output.stdout);
    }
    let failure = String::from_utf8_lossy(&output.stderr);
    Err(SynthesisError::Failed(format!(
        "media-speech failed: {}",
        failure.trim()
    )))
}

fn spawn(program: &Path, arguments: &[&str]) -> Result<Child, SynthesisError> {
    Command::new(program)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| SynthesisError::Failed(format!("media-speech failed to start: {error}")))
}
