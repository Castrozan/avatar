use serde_json::{Value, json};

#[must_use]
pub fn needs_seat_lookup(command: &Value) -> bool {
    command.get("type") == Some(&json!("speak")) && !has_output(command)
}

fn has_output(command: &Value) -> bool {
    command
        .get("output")
        .and_then(Value::as_str)
        .is_some_and(|output| !output.is_empty())
}

#[must_use]
pub fn with_speech_output(command: Value, seat_attached: bool) -> Value {
    if !needs_seat_lookup(&command) {
        return command;
    }
    let mut chosen = command;
    chosen["output"] = json!(if seat_attached { "remote" } else { "both" });
    chosen
}
