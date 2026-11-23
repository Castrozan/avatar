use crate::errors::AudioGraphError;
use crate::pulse::control::{PulseControl, PulseError};

pub const VIRTUAL_MICROPHONE_SINK: &str = "avatar-virtual-mic";
const VIRTUAL_MICROPHONE_DESCRIPTION: &str = "Avatar Virtual Microphone";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtualMicrophoneStart {
    Created,
    AlreadyRunning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtualMicrophoneStop {
    Removed,
    NotFound,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VirtualMicrophoneStatus {
    Active { monitor_sources: Vec<String> },
    Inactive,
}

pub fn start(pulse: &mut impl PulseControl) -> Result<VirtualMicrophoneStart, AudioGraphError> {
    match load_sink(pulse) {
        Ok(_) => Ok(VirtualMicrophoneStart::Created),
        Err(error) => already_running_or(pulse, error),
    }
}

fn load_sink(pulse: &mut impl PulseControl) -> Result<String, PulseError> {
    let sink_argument = format!("sink_name={VIRTUAL_MICROPHONE_SINK}");
    let description_argument =
        format!("sink_properties=device.description={VIRTUAL_MICROPHONE_DESCRIPTION}");
    pulse.run(&[
        "load-module",
        "module-null-sink",
        &sink_argument,
        &description_argument,
    ])
}

fn already_running_or(
    pulse: &mut impl PulseControl,
    error: PulseError,
) -> Result<VirtualMicrophoneStart, AudioGraphError> {
    if sink_listed(pulse)? {
        Ok(VirtualMicrophoneStart::AlreadyRunning)
    } else {
        Err(error.into())
    }
}

pub fn stop(pulse: &mut impl PulseControl) -> Result<VirtualMicrophoneStop, AudioGraphError> {
    let listing = pulse.run(&["list", "short", "modules"])?;
    match module_identifier(&listing) {
        Some(identifier) => {
            pulse.run(&["unload-module", &identifier])?;
            Ok(VirtualMicrophoneStop::Removed)
        }
        None => Ok(VirtualMicrophoneStop::NotFound),
    }
}

pub fn status(pulse: &mut impl PulseControl) -> Result<VirtualMicrophoneStatus, AudioGraphError> {
    if !sink_listed(pulse)? {
        return Ok(VirtualMicrophoneStatus::Inactive);
    }
    let sources = pulse.run(&["list", "short", "sources"])?;
    let monitor_sources = lines_naming_virtual_microphone(&sources);
    Ok(VirtualMicrophoneStatus::Active { monitor_sources })
}

fn sink_listed(pulse: &mut impl PulseControl) -> Result<bool, AudioGraphError> {
    let sinks = pulse.run(&["list", "short", "sinks"])?;
    Ok(!lines_naming_virtual_microphone(&sinks).is_empty())
}

fn lines_naming_virtual_microphone(listing: &str) -> Vec<String> {
    listing
        .lines()
        .filter(|line| line.contains(VIRTUAL_MICROPHONE_SINK))
        .map(str::to_owned)
        .collect()
}

fn module_identifier(listing: &str) -> Option<String> {
    listing
        .lines()
        .find(|line| line.contains(VIRTUAL_MICROPHONE_SINK))
        .and_then(|line| line.split_whitespace().next())
        .map(str::to_owned)
}
