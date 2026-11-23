use std::time::Duration;

use serde_json::Value;

use crate::process::loopback_get;

const PROBE_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Clone, Copy, Debug)]
pub struct Endpoints {
    pub control_port: u16,
    pub seat_port: u16,
}

impl Default for Endpoints {
    fn default() -> Self {
        Self {
            control_port: 8766,
            seat_port: 8769,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct HealthProbe {
    endpoints: Endpoints,
}

impl HealthProbe {
    #[must_use]
    pub const fn new(endpoints: Endpoints) -> Self {
        Self { endpoints }
    }

    #[must_use]
    pub fn control_server_ready(self) -> bool {
        self.control_field("status") == Some(Value::String("ok".to_owned()))
    }

    #[must_use]
    pub fn renderer_ready(self) -> bool {
        self.control_field("rendererConnected") == Some(Value::Bool(true))
    }

    #[must_use]
    pub fn remote_seat_listening(self) -> bool {
        loopback_get(self.endpoints.seat_port, "/health", PROBE_TIMEOUT)
            .is_some_and(|response| response.status().as_u16() == 200)
    }

    fn control_field(self, name: &str) -> Option<Value> {
        let response = loopback_get(self.endpoints.control_port, "/health", PROBE_TIMEOUT)?;
        let body = response.error_for_status().ok()?.bytes().ok()?;
        let document: Value = serde_json::from_slice(&body).ok()?;
        document.get(name).cloned()
    }
}
