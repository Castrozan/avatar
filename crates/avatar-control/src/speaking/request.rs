use serde_json::Value;

const DEFAULT_EMOTION: &str = "neutral";
const DEFAULT_OUTPUT: &str = "speakers";

#[derive(Debug, Clone, PartialEq)]
pub struct SpeakRequest {
    pub id: Value,
    pub text: String,
    pub emotion: Option<String>,
    pub output: Value,
    pub voice: Option<String>,
    pub model: Option<String>,
}

impl SpeakRequest {
    #[must_use]
    pub fn from_command(command: &Value, now_ms: u64) -> Option<Self> {
        let text = speakable_text(command)?;
        Some(Self {
            id: field_or(command, "id", || Value::from(now_ms.to_string())),
            text: text.to_owned(),
            emotion: defaulted_text(command, "emotion", DEFAULT_EMOTION),
            output: field_or(command, "output", || Value::from(DEFAULT_OUTPUT)),
            voice: optional_text(command, "voice"),
            model: optional_text(command, "model"),
        })
    }

    #[must_use]
    pub fn output_id(&self, chunk_index: usize) -> String {
        format!("{}-{chunk_index}", identifier_text(&self.id))
    }
}

fn speakable_text(command: &Value) -> Option<&str> {
    command
        .get("text")
        .and_then(Value::as_str)
        .filter(|text| !is_blank(text))
}

fn is_blank(text: &str) -> bool {
    text.trim_matches(|character: char| character.is_whitespace() || character == '\u{feff}')
        .is_empty()
}

fn identifier_text(id: &Value) -> String {
    match id {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

fn field_or(command: &Value, field: &str, fallback: impl FnOnce() -> Value) -> Value {
    command.get(field).cloned().unwrap_or_else(fallback)
}

fn defaulted_text(command: &Value, field: &str, default: &str) -> Option<String> {
    match command.get(field) {
        None => Some(default.to_owned()),
        Some(value) => value.as_str().map(str::to_owned),
    }
}

fn optional_text(command: &Value, field: &str) -> Option<String> {
    command
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_owned)
}
