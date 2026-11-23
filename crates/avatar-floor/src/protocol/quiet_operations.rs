use serde_json::Value;

pub(super) fn handle(name: &str, _request: &Value) -> Result<Value, String> {
    Err(format!("unknown quiet operation {name}"))
}
