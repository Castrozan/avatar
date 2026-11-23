use serde_json::Value;

pub(super) struct Heard {
    pub(super) authorization: String,
    pub(super) messages: Vec<Value>,
}

impl Heard {
    pub(super) fn sent(&self, kind: &str) -> bool {
        self.messages.iter().any(|message| message["type"] == kind)
    }

    pub(super) fn delivered(&self, task: &str, content: &str) -> bool {
        let matches =
            |message: &Value| message["delegation_id"] == task && message["content"] == content;
        self.messages.iter().any(matches)
    }
}
