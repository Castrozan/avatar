use avatar_audio_graph::effects::names::{Channel, MEETING_SOURCE};
use avatar_audio_graph::errors::AudioGraphError;
use avatar_audio_graph::ownership::{AudioPhase, observe};
use avatar_audio_graph::pulse::control::{PulseControl, PulseError};
use avatar_audio_graph::pulse::listing::parse_modules;
use avatar_audio_graph::state::record::AudioState;
use serde_json::{Map, Value, json};

const REPORTED_FIELDS: [&str; 3] = ["name", "mute", "volume"];

pub fn describe(
    pulse: &mut impl PulseControl,
    state: &AudioState,
) -> Result<Value, AudioGraphError> {
    let modules = parse_modules(&pulse.run(&["list", "short", "modules"])?);
    let phase = observe(state, &modules);
    let sinks = listed_sinks(pulse)?;
    Ok(json!({
        "running": matches!(phase, AudioPhase::RunningDetached | AudioPhase::RunningAttached),
        "phase": phase_name(phase),
        "microphone": state.microphone.as_ref().map(|microphone| microphone.source.clone()),
        "meeting_source": MEETING_SOURCE,
        "channels": channels(&sinks),
    }))
}

const fn phase_name(phase: AudioPhase) -> &'static str {
    match phase {
        AudioPhase::Absent => "absent",
        AudioPhase::RunningDetached => "running-detached",
        AudioPhase::RunningAttached => "running-attached",
        AudioPhase::Orphaned => "orphaned",
    }
}

const fn channel_name(channel: Channel) -> &'static str {
    match channel {
        Channel::Agent => "agent",
        Channel::Human => "human",
    }
}

fn listed_sinks(pulse: &mut impl PulseControl) -> Result<Vec<Value>, AudioGraphError> {
    let output = pulse.run(&["--format=json", "list", "sinks"])?;
    match serde_json::from_str(&output) {
        Ok(Value::Array(sinks)) => Ok(sinks),
        _ => Err(PulseError::new("pactl did not list sinks as a JSON array").into()),
    }
}

fn channels(sinks: &[Value]) -> Map<String, Value> {
    Channel::ALL
        .into_iter()
        .flat_map(|channel| reported_sinks(sinks, channel))
        .collect()
}

fn reported_sinks(sinks: &[Value], channel: Channel) -> impl Iterator<Item = (String, Value)> {
    let named = sinks
        .iter()
        .filter(move |sink| sink.get("name").and_then(Value::as_str) == Some(channel.sink_name()));
    named.map(move |sink| (channel_name(channel).to_owned(), reported_fields(sink)))
}

fn reported_fields(sink: &Value) -> Value {
    let fields = REPORTED_FIELDS.iter().map(|field| {
        (
            (*field).to_owned(),
            sink.get(*field).cloned().unwrap_or(Value::Null),
        )
    });
    Value::Object(fields.collect())
}
