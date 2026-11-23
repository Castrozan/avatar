use serde_json::{Map, Value};

pub(crate) type Field = (&'static str, Option<Value>);

pub(crate) const SERVER_VERSION: &str = "1.0.0";

pub(crate) fn message(kind: &str, fields: Vec<(&str, Option<Value>)>) -> Value {
    let present = fields
        .into_iter()
        .filter_map(|(key, value)| value.map(|field| (key.to_owned(), field)));
    let typed = std::iter::once(("type".to_owned(), Value::from(kind))).chain(present);
    Value::Object(typed.collect::<Map<String, Value>>())
}

pub(crate) fn error(text: &str, now_ms: u64) -> Value {
    message(
        "error",
        vec![
            ("error", Some(Value::from(text))),
            ("timestamp", Some(Value::from(now_ms))),
        ],
    )
}

pub(crate) fn identify_ack(role: &str) -> Value {
    message(
        "identifyAck",
        vec![
            ("role", Some(Value::from(role))),
            ("status", Some(Value::from("connected"))),
            ("serverVersion", Some(Value::from(SERVER_VERSION))),
        ],
    )
}

pub(crate) fn describe(value: Option<&Value>) -> String {
    match value {
        None => "undefined".to_owned(),
        Some(Value::String(text)) => text.clone(),
        Some(other) => other.to_string(),
    }
}
