use serde_json::Value;

pub(super) fn cases(fixture: &str, key: &str) -> Result<Vec<Value>, serde_json::Error> {
    let parsed: Value = serde_json::from_str(fixture)?;
    Ok(parsed[key].as_array().cloned().unwrap_or_default())
}

pub(super) fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}

pub(super) fn optional_text(value: &Value) -> Option<String> {
    value.as_str().map(str::to_owned)
}

pub(super) fn number(value: &Value) -> f64 {
    value.as_f64().unwrap_or_default()
}
