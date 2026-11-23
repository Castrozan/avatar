use super::{AUDIO_COMMAND_TIMEOUT_SECONDS, StartEdges};
use crate::local::SeatFailure;
use crate::meetings::run_checked;
use crate::support::words::joined_words;

const CALL_CHANNELS: [&str; 2] = ["agent", "human"];

pub(crate) fn audio_command(arguments: &[&str]) -> Vec<String> {
    joined_words(&[&["avatar-audio"], arguments])
}

pub(crate) fn run_audio(edges: &StartEdges<'_>, arguments: &[&str]) -> Result<(), SeatFailure> {
    let command = audio_command(arguments);
    run_checked(
        edges.processes,
        &command,
        AUDIO_COMMAND_TIMEOUT_SECONDS,
        true,
    )?;
    Ok(())
}

pub(crate) fn open_call_channels(edges: &StartEdges<'_>) -> Result<(), SeatFailure> {
    for channel in CALL_CHANNELS {
        run_audio(edges, &["mute", channel, "0"])?;
    }
    Ok(())
}
