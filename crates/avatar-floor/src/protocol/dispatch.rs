use serde_json::{Value, json};

use super::{claim_operations, floor_operations, quiet_operations};

#[must_use]
pub fn respond(line: &str) -> Value {
    match answer(line) {
        Ok(value) => json!({"ok": value}),
        Err(message) => json!({"error": message}),
    }
}

fn answer(line: &str) -> Result<Value, String> {
    let request: Value = serde_json::from_str(line).map_err(|error| error.to_string())?;
    let operation = request
        .get("op")
        .and_then(Value::as_str)
        .ok_or("missing op")?;
    let (group, name) = operation.split_once('.').ok_or("op needs a group prefix")?;
    route(group, name, &request)
}

fn route(group: &str, name: &str, request: &Value) -> Result<Value, String> {
    match group {
        "floor" => floor_operations::handle(name, request),
        "claim" => claim_operations::handle(name, request),
        "quiet" => quiet_operations::handle(name, request),
        other => Err(format!("unknown group {other}")),
    }
}
