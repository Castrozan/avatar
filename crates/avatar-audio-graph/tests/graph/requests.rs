use avatar_audio_graph::effects::names::Channel;
use avatar_audio_graph::graph::AudioRequest;

use super::fixture::StepSpec;

pub(super) fn request_for(step: &StepSpec) -> Result<AudioRequest, String> {
    match step.action.as_str() {
        "attach" => Ok(AudioRequest::Attach(text(step.source.as_ref())?)),
        "mute" | "volume" => level_request(step),
        action => plain_request(action),
    }
}

pub(super) fn plain_request(action: &str) -> Result<AudioRequest, String> {
    match action {
        "start" => Ok(AudioRequest::Start),
        "stop" => Ok(AudioRequest::Stop),
        "open" => Ok(AudioRequest::Open),
        "recover" => Ok(AudioRequest::Recover),
        "status" => Ok(AudioRequest::Status),
        other => Err(format!("unknown action {other}")),
    }
}

pub(super) fn level_request(step: &StepSpec) -> Result<AudioRequest, String> {
    let channel = channel(step)?;
    let request = match step.action.as_str() {
        "mute" => AudioRequest::mute(channel, integer(step.value.as_ref())?),
        _ => AudioRequest::volume(channel, percent(step)?),
    };
    request.map_err(|error| error.to_string())
}

pub(super) fn integer(field: Option<&String>) -> Result<i64, String> {
    text(field)?.parse().map_err(to_text)
}

pub(super) fn percent(step: &StepSpec) -> Result<i64, String> {
    text(step.value.as_ref())?
        .trim_end_matches('%')
        .parse()
        .map_err(to_text)
}

pub(super) fn channel(step: &StepSpec) -> Result<Channel, String> {
    match text(step.channel.as_ref())?.as_str() {
        "agent" => Ok(Channel::Agent),
        "human" => Ok(Channel::Human),
        other => Err(format!("unknown channel {other}")),
    }
}

pub(super) fn text(field: Option<&String>) -> Result<String, String> {
    field.cloned().ok_or_else(|| "missing field".to_owned())
}

pub(super) fn to_text(error: impl std::fmt::Display) -> String {
    error.to_string()
}
