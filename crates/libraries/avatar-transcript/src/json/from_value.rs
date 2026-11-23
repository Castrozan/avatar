use serde_json::{Map, Number, Value};

use super::{Json, JsonObject};

#[must_use]
pub fn from_value(value: &Value) -> Json {
    match value {
        Value::Null => Json::Null,
        Value::Bool(flag) => Json::Bool(*flag),
        Value::Number(number) => from_number(number),
        Value::String(text) => Json::Text(text.clone()),
        Value::Array(items) => Json::Array(items.iter().map(from_value).collect()),
        Value::Object(fields) => Json::Object(from_fields(fields)),
    }
}

fn from_number(number: &Number) -> Json {
    number.as_i64().map_or_else(
        || Json::Float(number.as_f64().unwrap_or(f64::NAN)),
        Json::Integer,
    )
}

fn from_fields(fields: &Map<String, Value>) -> JsonObject {
    let mut object = JsonObject::default();
    for (key, value) in fields {
        object.set(key.clone(), from_value(value));
    }
    object
}
