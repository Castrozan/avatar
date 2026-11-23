use avatar_transcript::json::{Json, JsonObject};
use serde_json::Value;

fn to_json(value: &Value) -> Json {
    match value {
        Value::Null => Json::Null,
        Value::Bool(flag) => Json::Bool(*flag),
        Value::Number(number) => number_to_json(number),
        Value::String(text) => Json::Text(text.clone()),
        Value::Array(items) => Json::Array(items.iter().map(to_json).collect()),
        Value::Object(fields) => Json::Object(object_to_json(fields)),
    }
}

fn number_to_json(number: &serde_json::Number) -> Json {
    match number.as_i64() {
        Some(integer) => Json::Integer(integer),
        None => Json::Float(number.as_f64().unwrap_or(f64::NAN)),
    }
}

fn object_to_json(fields: &serde_json::Map<String, Value>) -> JsonObject {
    let mut object = JsonObject::default();
    for (key, value) in fields {
        object.set(key.clone(), to_json(value));
    }
    object
}

pub(super) fn details_of(pairs: &[(String, Value)]) -> JsonObject {
    let mut object = JsonObject::default();
    for (key, value) in pairs {
        object.set(key.clone(), to_json(value));
    }
    object
}
