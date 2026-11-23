use std::collections::HashMap;

use avatar_control::speaking::runtime::Platform;
use avatar_control::speech::spoken_vowels::{SpokenVowel, VowelShape};
use avatar_control::synthesis::contract::SpeechSource;
use serde_json::Value;

use super::script::ChannelFaults;

pub(crate) fn strings(scenario: &Value, key: &str) -> Vec<String> {
    scenario
        .get(key)
        .and_then(Value::as_array)
        .map_or_else(Vec::new, |items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_owned))
                .collect()
        })
}

pub(crate) fn faults(scenario: &Value) -> ChannelFaults {
    ChannelFaults {
        open: text(scenario, "openFails"),
        release: text(scenario, "releaseFails"),
    }
}

pub(crate) fn array(scenario: &Value, key: &str) -> Vec<Value> {
    scenario
        .get(key)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

pub(crate) fn text(scenario: &Value, key: &str) -> Option<String> {
    scenario.get(key)?.as_str().map(str::to_owned)
}

pub(crate) fn failures(scenario: &Value) -> HashMap<String, String> {
    let table = scenario.get("failGenerate").and_then(Value::as_object);
    table.map_or_else(HashMap::new, |table| {
        let pairs = table
            .iter()
            .filter_map(|(key, value)| Some((key.clone(), value.as_str()?.to_owned())));
        pairs.collect()
    })
}

pub(crate) fn shape_of(name: &str) -> VowelShape {
    match name {
        "aa" => VowelShape::Aa,
        "ee" => VowelShape::Ee,
        "ih" => VowelShape::Ih,
        "oh" => VowelShape::Oh,
        _ => VowelShape::Ou,
    }
}

pub(crate) fn source_of(name: &str) -> SpeechSource {
    match name {
        "elevenlabs" => SpeechSource::ElevenLabs,
        "piper" => SpeechSource::Piper,
        _ => SpeechSource::Kokoro,
    }
}

pub(crate) fn vowel_of(vowel: &Value) -> SpokenVowel {
    SpokenVowel {
        shape: shape_of(vowel["shape"].as_str().unwrap_or_default()),
        start: vowel["start"].as_f64().unwrap_or_default(),
        end: vowel["end"].as_f64().unwrap_or_default(),
    }
}

pub(crate) fn platform_of(scenario: &Value) -> Platform {
    match scenario.get("platform").and_then(Value::as_str) {
        Some("darwin") => Platform::Darwin,
        _ => Platform::Linux,
    }
}
