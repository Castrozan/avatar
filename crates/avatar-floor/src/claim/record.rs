use serde_json::Value;

const CLAIM_SECONDS: f64 = 10.0;

pub(super) struct RecordedClaim {
    room: Option<String>,
    claimed_at: f64,
}

impl RecordedClaim {
    fn from_record(record: &Value) -> Self {
        Self {
            room: record
                .get("room")
                .and_then(Value::as_str)
                .map(str::to_owned),
            claimed_at: record
                .get("claimed_at")
                .and_then(Value::as_f64)
                .unwrap_or(0.0),
        }
    }

    pub(super) fn other_holder(&self, room: &str) -> Option<&str> {
        self.room
            .as_deref()
            .filter(|holder| !holder.is_empty() && *holder != room)
    }
}

pub(super) fn recorded_claim(text: &str) -> RecordedClaim {
    match serde_json::from_str::<Value>(text) {
        Ok(record) => RecordedClaim::from_record(&record),
        Err(_) => RecordedClaim {
            room: None,
            claimed_at: 0.0,
        },
    }
}

#[must_use]
pub fn held_by_another_room(text: &str, room: &str, now: f64) -> bool {
    let claim = recorded_claim(text);
    claim.other_holder(room).is_some() && now - claim.claimed_at < CLAIM_SECONDS
}
