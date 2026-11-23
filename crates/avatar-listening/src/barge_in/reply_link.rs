pub mod control_socket;

use serde_json::Value;

const SPEECH_ACTIVITY: &str = "speechActivity";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerNotice {
    SpeechActivity { speaking: bool },
    Other,
}

impl ServerNotice {
    #[must_use]
    pub fn from_message(message: &Value) -> Option<Self> {
        parse_notice(message)
    }

    #[must_use]
    pub fn speaking_state(&self) -> Option<bool> {
        match self {
            Self::SpeechActivity { speaking } => Some(*speaking),
            Self::Other => None,
        }
    }
}

#[derive(Debug, Default)]
pub struct ReplyLinkState {
    speaking: bool,
}

impl ReplyLinkState {
    #[must_use]
    pub fn speaking(&self) -> bool {
        self.speaking
    }

    pub fn hear_notice(&mut self, notice: &ServerNotice) {
        self.speaking = notice.speaking_state().unwrap_or(self.speaking);
    }

    pub fn lost_server(&mut self) {
        self.speaking = false;
    }
}

fn parse_notice(message: &Value) -> Option<ServerNotice> {
    match message.get("type").and_then(Value::as_str) {
        Some(SPEECH_ACTIVITY) => Some(ServerNotice::SpeechActivity {
            speaking: is_truthy(message.get("speaking")?),
        }),
        _ => Some(ServerNotice::Other),
    }
}

#[must_use]
fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(flag) => *flag,
        Value::Number(number) => number.as_f64().is_some_and(|float| float != 0.0),
        Value::String(text) => !text.is_empty(),
        Value::Array(items) => !items.is_empty(),
        Value::Object(fields) => !fields.is_empty(),
    }
}
