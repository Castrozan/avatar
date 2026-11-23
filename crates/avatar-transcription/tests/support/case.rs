use super::mock::RecordedRequest;
use avatar_transcription::TranscriptionError;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub(crate) struct Case {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) responses: Vec<(u16, Value)>,
    #[serde(default)]
    pub(crate) calls: Vec<Call>,
    #[serde(default)]
    pub(crate) pcm_hex: String,
    #[serde(default)]
    pub(crate) roster_text: Option<String>,
    #[serde(default)]
    pub(crate) roster_missing: bool,
    #[serde(default)]
    pub(crate) vocabulary_text: Option<String>,
    #[serde(default)]
    pub(crate) closed_port: bool,
    pub(crate) outcomes: Vec<Outcome>,
    pub(crate) requests: Vec<Expected>,
    pub(crate) remaining_responses: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Operation {
    Cloud,
    Fallback,
    Hex,
    Keyterms,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Call {
    pub(crate) op: Operation,
    #[serde(default)]
    pub(crate) language: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Outcome {
    #[serde(default)]
    pub(crate) value: Option<Value>,
    #[serde(default)]
    pub(crate) error: Option<ExpectedError>,
    pub(crate) stderr: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ExpectedError {
    #[serde(rename = "type")]
    pub(crate) kind: String,
    pub(crate) message: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Expected {
    pub(crate) method: String,
    pub(crate) path: String,
    pub(crate) headers: Vec<(String, String)>,
    pub(crate) body_hex: String,
}

pub(crate) type Actual = (Result<Value, TranscriptionError>, Vec<String>);

pub(crate) struct Replay {
    pub(crate) outcomes: Vec<Actual>,
    pub(crate) requests: Vec<RecordedRequest>,
    pub(crate) remaining: usize,
}

pub(crate) fn decode_hex(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .filter_map(|index| {
            text.get(index..index + 2)
                .and_then(|pair| u8::from_str_radix(pair, 16).ok())
        })
        .collect()
}
