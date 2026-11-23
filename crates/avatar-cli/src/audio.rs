use std::io::{self, Write};
use std::process::ExitCode;

use avatar_adapters::commands::{AudioEnvironment, Programs, run_audio, runtime_directory};
use avatar_audio_graph::effects::names::Channel;
use avatar_audio_graph::errors::AudioGraphError;
use avatar_audio_graph::graph::AudioRequest;
use clap::{Args, Subcommand, ValueEnum};
use nix::unistd::getuid;

const SEAT_PORT: u16 = 8769;

#[derive(Args)]
pub(crate) struct AudioOptions {
    #[command(subcommand)]
    pub(crate) action: AudioAction,
}

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum ChannelChoice {
    Agent,
    Human,
}

#[derive(Subcommand)]
pub(crate) enum AudioAction {
    #[command(about = "Build the avatar audio graph")]
    Start,
    #[command(about = "Tear the avatar audio graph down")]
    Stop,
    #[command(about = "Report the avatar audio graph")]
    Status,
    #[command(about = "Attach the default microphone and unmute the inputs")]
    Open,
    #[command(about = "Rebuild an orphaned audio graph")]
    Recover,
    #[command(about = "Attach a human input source")]
    Attach { source: String },
    #[command(about = "Mute (1) or unmute (0) a channel")]
    Mute {
        channel: ChannelChoice,
        #[arg(allow_negative_numbers = true)]
        value: i64,
    },
    #[command(about = "Set a channel volume from 0 to 100")]
    Volume {
        channel: ChannelChoice,
        #[arg(allow_negative_numbers = true)]
        value: i64,
    },
}

impl AudioAction {
    fn request(&self) -> Result<AudioRequest, AudioGraphError> {
        match self {
            Self::Start => Ok(AudioRequest::Start),
            Self::Stop => Ok(AudioRequest::Stop),
            Self::Status => Ok(AudioRequest::Status),
            Self::Open => Ok(AudioRequest::Open),
            Self::Recover => Ok(AudioRequest::Recover),
            Self::Attach { source } => Ok(AudioRequest::Attach(source.clone())),
            Self::Mute { channel, value } => AudioRequest::mute(channel.channel(), *value),
            Self::Volume { channel, value } => AudioRequest::volume(channel.channel(), *value),
        }
    }
}

impl ChannelChoice {
    const fn channel(self) -> Channel {
        match self {
            Self::Agent => Channel::Agent,
            Self::Human => Channel::Human,
        }
    }
}

pub(crate) fn environment() -> AudioEnvironment {
    let base = std::env::var_os("XDG_RUNTIME_DIR");
    AudioEnvironment {
        programs: Programs::on_path(),
        runtime_directory: runtime_directory(base, getuid().as_raw()),
        seat_port: SEAT_PORT,
    }
}

pub(crate) fn run(action: &AudioAction) -> io::Result<ExitCode> {
    let state = run_audio(&environment(), || action.request())
        .map_err(|error| io::Error::other(error.to_string()))?;
    writeln!(io::stdout().lock(), "{state}")?;
    Ok(ExitCode::SUCCESS)
}
