use std::collections::HashSet;

use serde_json::{Value, json};

use super::request::SpeakRequest;
use super::timing::SpeechTiming;
use crate::speech::spoken_vowels::SpokenVowel;
use crate::speech::turn::TurnStatus;
use crate::synthesis::contract::SpeechSource;

#[must_use]
pub fn speaking_started(request: &SpeakRequest, chunk: &str, vowels: &[SpokenVowel]) -> Value {
    json!({
        "type": "startSpeaking",
        "id": request.id,
        "emotion": request.emotion,
        "text": chunk,
        "vowels": vowels.iter().map(vowel_json).collect::<Vec<Value>>(),
    })
}

fn vowel_json(vowel: &SpokenVowel) -> Value {
    json!({"shape": vowel.shape.name(), "start": vowel.start, "end": vowel.end})
}

#[must_use]
pub fn speech_ended(request: &SpeakRequest) -> Value {
    json!({"type": "speechEnded", "id": request.id})
}

#[must_use]
pub fn speak_ack(
    request: &SpeakRequest,
    status: TurnStatus,
    sources: &[SpeechSource],
    timing: &SpeechTiming,
) -> Value {
    let ack = json!({
        "type": "speakAck",
        "id": request.id,
        "output": request.output,
        "status": status_name(status),
        "timing": timing.to_json(),
    });
    with_source(ack, sources)
}

fn with_source(mut ack: Value, sources: &[SpeechSource]) -> Value {
    if let Some(joined) = spoken_sources(sources) {
        ack["source"] = Value::from(joined);
    }
    ack
}

#[must_use]
pub const fn status_name(status: TurnStatus) -> &'static str {
    match status {
        TurnStatus::Finished => "finished",
        TurnStatus::Interrupted => "interrupted",
        TurnStatus::Cancelled => "cancelled",
        TurnStatus::Failed => "failed",
    }
}

#[must_use]
pub const fn source_name(source: SpeechSource) -> &'static str {
    match source {
        SpeechSource::Kokoro => "kokoro",
        SpeechSource::ElevenLabs => "elevenlabs",
        SpeechSource::Piper => "piper",
    }
}

fn spoken_sources(sources: &[SpeechSource]) -> Option<String> {
    let mut seen = HashSet::new();
    let names: Vec<&str> = sources
        .iter()
        .map(|source| source_name(*source))
        .filter(|name| seen.insert(*name))
        .collect();
    (!names.is_empty()).then(|| names.join("+"))
}
