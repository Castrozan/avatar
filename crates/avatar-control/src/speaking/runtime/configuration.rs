use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::sync::Arc;

use super::channel::agent_channel_for;
use super::clock::SystemClock;
use super::platform::Platform;
use super::players::{PlayerPrograms, SinkPlayers};
use super::port::{SpeakingPort, start_speech_runtime};
use super::services::SpeechServices;
use crate::synthesis::assembly::speech_generator_from_environment;
use crate::synthesis::contract::SynthesisError;

const AVATAR_AUDIO_PROGRAM: &str = "avatar-audio";
const PROCESS_DIRECTORY: &str = "/proc/self";

#[derive(Debug, Clone)]
pub struct SpeechRuntimeConfiguration {
    pub platform: Platform,
    pub avatar_audio_program: PathBuf,
    pub players: PlayerPrograms,
    pub pulse_runtime_directory: String,
}

impl SpeechRuntimeConfiguration {
    pub fn from_environment() -> io::Result<Self> {
        let platform = Platform::current();
        Ok(Self {
            platform,
            avatar_audio_program: PathBuf::from(AVATAR_AUDIO_PROGRAM),
            players: PlayerPrograms::default(),
            pulse_runtime_directory: pulse_runtime_directory(platform)?,
        })
    }

    #[must_use]
    pub fn services(&self) -> SpeechServices {
        SpeechServices {
            clock: Arc::new(SystemClock),
            channel: agent_channel_for(self.platform, self.avatar_audio_program.clone()),
            players: Arc::new(SinkPlayers::new(
                &self.players,
                &self.pulse_runtime_directory,
            )),
            platform: self.platform,
        }
    }
}

fn pulse_runtime_directory(platform: Platform) -> io::Result<String> {
    match platform {
        Platform::Darwin => Ok(String::new()),
        Platform::Linux => {
            let uid = std::fs::metadata(PROCESS_DIRECTORY)?.uid();
            Ok(format!("/run/user/{uid}"))
        }
    }
}

pub async fn speech_port_from_environment() -> Result<SpeakingPort, SynthesisError> {
    let configuration = SpeechRuntimeConfiguration::from_environment()?;
    let stack = speech_generator_from_environment().await?;
    Ok(start_speech_runtime(stack, configuration.services()))
}
