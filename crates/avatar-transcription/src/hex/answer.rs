use super::request::refusal_code_text;
use crate::error::{Outcome, missing_field};
use crate::segment::clean_transcript;
use serde_json::Value;

pub(super) fn accepted(status: u16, answer: Value) -> Outcome<Value> {
    if status == 200 {
        return Ok(answer);
    }
    Err(crate::error::TranscriptionError::Refused {
        code: refusal_code_text(&answer),
    })
}

pub(super) fn transcript_of(answer: &Value) -> Outcome<String> {
    let transcript = answer
        .get("transcript")
        .ok_or_else(|| missing_field("transcript"))?;
    transcript.as_str().map(clean_transcript).ok_or_else(|| {
        crate::error::TranscriptionError::MalformedBody("transcript is not text".to_owned())
    })
}
