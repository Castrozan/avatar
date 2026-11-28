use serde_json::Value;

pub(super) struct Heard {
    pub(super) api_keys: Vec<String>,
    pub(super) messages: Vec<Value>,
}

impl Heard {
    pub(super) fn streamed_audio(&self) -> bool {
        self.messages
            .iter()
            .any(|message| message["user_audio_chunk"].is_string())
    }

    pub(super) fn delivered(&self, answer: &str) -> bool {
        let text = format!("Agent answer: {answer}");
        let matches =
            |message: &Value| message["type"] == "user_message" && message["text"] == text;
        self.messages.iter().any(matches)
    }
}
