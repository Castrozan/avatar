use serde_json::{Value, json};

#[must_use]
pub fn status_report(is_open: bool, live: bool, seat: Option<&str>, last_heard: u64) -> Value {
    json!({"open": is_open, "live": live, "seat": seat, "last_heard": last_heard})
}

#[must_use]
pub fn closed_report() -> Value {
    json!({"open": false})
}

#[must_use]
pub fn room_held_by_report(servant: &str) -> Value {
    json!({"open": true, "room_held_by": servant})
}

#[must_use]
pub fn floor_returned_report(servant: &str) -> Value {
    json!({"open": true, "floor_returned_to": servant})
}
