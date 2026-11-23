use serde_json::Value;

use crate::session::agent_commands;
use crate::session::client::Outbound;
use crate::session::hub::HealthSnapshot;
use crate::session::hub::Hub;

impl Hub {
    #[must_use]
    pub fn health(&self, now_ms: u64) -> HealthSnapshot {
        HealthSnapshot {
            uptime_seconds: self.avatar.uptime_seconds(now_ms),
            state: self.speech_phase.clone(),
            agent_connected: self.registry.has_agents(),
            renderer_connected: self.registry.has_renderer(),
        }
    }

    #[must_use]
    pub fn broadcast_voice_level(&mut self, level: f64, now_ms: u64) -> Vec<Outbound> {
        agent_commands::audio_level(self, Value::from(level), now_ms).outbound
    }
}
