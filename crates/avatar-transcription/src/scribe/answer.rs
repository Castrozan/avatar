use crate::error::{TranscriptionError, python_display};
use serde_json::Value;

pub(super) fn text_of(answer: &Value) -> Result<&str, TranscriptionError> {
    let text = answer
        .get("text")
        .ok_or_else(|| TranscriptionError::MissingField("text".to_owned()))?;
    text.as_str()
        .ok_or_else(|| TranscriptionError::MalformedBody("text is not a string".to_owned()))
}

#[must_use]
pub(super) fn language_code_of(answer: &Value) -> Option<&str> {
    answer.get("language_code").and_then(Value::as_str)
}

#[must_use]
pub(super) fn probability_text(answer: &Value) -> String {
    python_display(answer.get("language_probability"))
}
