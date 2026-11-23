use serde_json::Value;

pub(super) fn cases(fixture: &str, key: &str) -> Result<Vec<Value>, serde_json::Error> {
    let parsed: Value = serde_json::from_str(fixture)?;
    Ok(parsed[key].as_array().cloned().unwrap_or_default())
}

pub(super) fn text(value: &Value) -> String {
    value.as_str().unwrap_or_default().to_owned()
}
