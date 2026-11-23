use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Run(pub(crate) i32, pub(crate) String, pub(crate) Option<String>);

pub(crate) fn run_call(command: &[String], timeout: u64, capture: bool) -> Value {
    json!(format!(
        "$ {}|{timeout}|0{}",
        command.join(" "),
        u8::from(capture)
    ))
}

pub(crate) fn spawn_call(command: &[String]) -> Value {
    json!({"kind": "spawn", "command": command})
}

pub(crate) fn sleep_call(seconds: f64) -> Value {
    json!(format!("S:{seconds}"))
}
