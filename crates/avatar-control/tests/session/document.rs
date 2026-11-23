use std::error::Error;

use serde_json::Value;

pub(crate) fn str_field<'value>(
    value: &'value Value,
    key: &str,
) -> Result<&'value str, Box<dyn Error>> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing string field {key}").into())
}

pub(crate) fn bool_field(value: &Value, key: &str) -> Result<bool, Box<dyn Error>> {
    value
        .get(key)
        .and_then(Value::as_bool)
        .ok_or_else(|| format!("missing boolean field {key}").into())
}

pub(crate) fn u64_field(value: &Value, key: &str) -> Result<u64, Box<dyn Error>> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("missing integer field {key}").into())
}

pub(crate) fn array_field<'value>(
    value: &'value Value,
    key: &str,
) -> Result<&'value Vec<Value>, Box<dyn Error>> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("missing array field {key}").into())
}

pub(crate) fn string_list(value: &Value, key: &str) -> Result<Vec<String>, Box<dyn Error>> {
    array_field(value, key)?
        .iter()
        .map(|item| {
            item.as_str()
                .map(str::to_owned)
                .ok_or_else(|| "voice names must be strings".into())
        })
        .collect()
}
