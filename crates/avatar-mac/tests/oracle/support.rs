use std::fmt::Write;

use avatar_mac::frame::Frame;
use serde_json::{Map, Value, json};

pub(crate) fn parse(text: &str) -> Value {
    serde_json::from_str(text).unwrap_or(Value::Null)
}

fn normalized_fields(fields: &Map<String, Value>) -> Value {
    let pairs = fields.iter();
    Value::Object(
        pairs
            .map(|(key, field)| (key.clone(), normalized(field)))
            .collect(),
    )
}

pub(crate) fn normalized(value: &Value) -> Value {
    match value {
        Value::Number(number) => Value::from(number.as_f64()),
        Value::Array(items) => Value::Array(items.iter().map(normalized).collect()),
        Value::Object(fields) => normalized_fields(fields),
        other => other.clone(),
    }
}

pub(crate) fn assert_same(left: &Value, right: &Value) {
    assert_eq!(normalized(left), normalized(right), "{right}");
}

pub(crate) fn entries(value: &Value) -> Vec<Value> {
    value.as_array().cloned().unwrap_or_default()
}

pub(crate) fn each(cases: &Value, check: fn(&Value)) {
    entries(cases).iter().for_each(check);
}

pub(crate) fn hex_bytes(text: &str) -> Vec<u8> {
    let pairs = (0..text.len() / 2).map(|at| text.get(at * 2..at * 2 + 2));
    pairs
        .filter_map(|pair| u8::from_str_radix(pair?, 16).ok())
        .collect()
}

pub(crate) fn hex_text(bytes: &[u8]) -> String {
    let mut text = String::new();
    for byte in bytes {
        let _ = write!(text, "{byte:02x}");
    }
    text
}

pub(crate) fn frame_of(value: &Value) -> Frame {
    match value["binary"].as_str() {
        Some(hex) => Frame::Binary(hex_bytes(hex)),
        None => Frame::Text(value["text"].as_str().unwrap_or_default().to_owned()),
    }
}

pub(crate) fn frame_json(frame: &Frame) -> Value {
    match frame {
        Frame::Binary(bytes) => json!({ "binary": hex_text(bytes) }),
        Frame::Text(text) => json!({ "text": text }),
    }
}

pub(crate) fn strings(value: &Value) -> Vec<String> {
    let items = entries(value);
    items
        .iter()
        .filter_map(|item| item.as_str().map(str::to_owned))
        .collect()
}

pub(crate) fn bits(value: &Value) -> Option<u64> {
    value.as_f64().map(f64::to_bits)
}
