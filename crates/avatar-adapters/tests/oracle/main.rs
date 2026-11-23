mod json;
mod quote;
mod request;
mod rotation;
mod server;

use serde_json::Value;

const ORACLE: &str = include_str!("fixtures/oracle.json");

fn oracle() -> Value {
    serde_json::from_str(ORACLE).unwrap_or_default()
}

fn section(name: &str) -> Vec<Value> {
    oracle()[name].as_array().cloned().unwrap_or_default()
}

fn text(value: &Value) -> &str {
    value.as_str().unwrap_or_default()
}
