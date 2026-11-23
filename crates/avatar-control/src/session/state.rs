use serde_json::{Value, json};

pub(crate) struct AvatarState {
    pub(crate) expression: Value,
    pub(crate) idle_mode: Value,
    pub(crate) intensity: Value,
    pub(crate) character: Option<String>,
    started_at_ms: u64,
}

impl AvatarState {
    pub(crate) fn new(started_at_ms: u64) -> Self {
        Self {
            expression: json!("neutral"),
            idle_mode: json!("breathing"),
            intensity: json!(1),
            character: None,
            started_at_ms,
        }
    }

    pub(crate) fn apply_expression(&mut self, expression: Value, intensity: Value) {
        self.expression = expression;
        self.intensity = intensity;
    }

    pub(crate) fn apply_idle_mode(&mut self, idle_mode: Value) {
        self.idle_mode = idle_mode;
    }

    pub(crate) fn apply_character(&mut self, character: Option<&str>) {
        self.character = character.map(str::to_owned);
    }

    pub(crate) fn uptime_seconds(&self, now_ms: u64) -> u64 {
        now_ms.saturating_sub(self.started_at_ms) / 1000
    }
}
