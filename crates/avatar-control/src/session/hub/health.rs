use serde_json::{Value, json};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthSnapshot {
    pub uptime_seconds: u64,
    pub state: String,
    pub agent_connected: bool,
    pub renderer_connected: bool,
}

impl HealthSnapshot {
    #[must_use]
    pub fn to_json(&self) -> Value {
        json!({
            "status": "ok",
            "uptime": self.uptime_seconds,
            "state": self.state,
            "agentConnected": self.agent_connected,
            "rendererConnected": self.renderer_connected,
        })
    }
}
