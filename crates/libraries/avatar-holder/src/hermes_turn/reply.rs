use crate::hermes_turn::HermesTurnError;
use serde_json::{Map, Value};

const PYTHON_LINE_BOUNDARIES: [char; 10] = [
    '\n', '\r', '\u{0b}', '\u{0c}', '\u{1c}', '\u{1d}', '\u{1e}', '\u{85}', '\u{2028}', '\u{2029}',
];

pub fn final_reply(turn_output: &str) -> Result<String, HermesTurnError> {
    turn_output
        .split(is_python_line_boundary)
        .rev()
        .map(result_text)
        .find_map(Result::transpose)
        .unwrap_or_else(|| Err(HermesTurnError::new("The Hermes turn produced no result")))
}

fn is_python_line_boundary(character: char) -> bool {
    PYTHON_LINE_BOUNDARIES.contains(&character)
}

fn result_text(line: &str) -> Result<Option<String>, HermesTurnError> {
    let Some(event) = result_event(line) else {
        return Ok(None);
    };
    let text = event
        .get("text")
        .and_then(Value::as_str)
        .ok_or_else(|| HermesTurnError::new("The Hermes result carried no text"))?;
    Ok(Some(text.to_owned()))
}

fn result_event(line: &str) -> Option<Map<String, Value>> {
    match serde_json::from_str::<Value>(line).ok()? {
        Value::Object(object) if object.get("type").and_then(Value::as_str) == Some("result") => {
            Some(object)
        }
        _ => None,
    }
}
