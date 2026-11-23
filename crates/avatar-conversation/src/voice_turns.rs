use serde_json::Value;

use crate::turn_taking::ACKNOWLEDGEMENT_TYPE;

const TIMING_DIVISOR_MILLISECONDS: f64 = 1000.0;
const COMMAND_PREFIX: [&str; 2] = ["avatar-command", "speak"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeechMoment {
    Received,
    SynthesisStarted,
    Synthesized,
    Playing,
}

impl SpeechMoment {
    pub const ALL: [Self; 4] = [
        Self::Received,
        Self::SynthesisStarted,
        Self::Synthesized,
        Self::Playing,
    ];

    #[must_use]
    pub const fn timing_key(self) -> &'static str {
        match self {
            Self::Received => "received",
            Self::SynthesisStarted => "synthesisStarted",
            Self::Synthesized => "synthesized",
            Self::Playing => "playing",
        }
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Received => "speech_received",
            Self::SynthesisStarted => "synthesis_started",
            Self::Synthesized => "synthesized",
            Self::Playing => "playing",
        }
    }
}

#[must_use]
pub fn acknowledgement(output: &str) -> Option<Value> {
    output.lines().find_map(|line| {
        serde_json::from_str::<Value>(line)
            .ok()
            .filter(is_acknowledgement)
    })
}

fn is_acknowledgement(response: &Value) -> bool {
    response.get("type").and_then(Value::as_str) == Some(ACKNOWLEDGEMENT_TYPE)
}

#[must_use]
pub fn speech_moments(response: &Value) -> Vec<(SpeechMoment, f64)> {
    let timing = response.get("timing").and_then(Value::as_object);
    SpeechMoment::ALL
        .into_iter()
        .filter_map(|moment| {
            let milliseconds = timing?.get(moment.timing_key())?.as_f64()?;
            Some((moment, milliseconds / TIMING_DIVISOR_MILLISECONDS))
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpeechRequest {
    pub text: String,
    pub emotion: String,
    pub voice: Option<String>,
}

#[must_use]
pub fn speech_command(request: &SpeechRequest, speech_output: &str) -> Vec<String> {
    let voice = request.voice.as_deref().filter(|value| !value.is_empty());
    let mut command = COMMAND_PREFIX.to_vec();
    command.extend([request.text.as_str(), request.emotion.as_str()]);
    command.push(speech_output);
    command
        .into_iter()
        .chain(voice)
        .map(str::to_owned)
        .collect()
}
