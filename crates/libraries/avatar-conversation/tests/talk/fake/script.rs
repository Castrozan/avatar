use avatar_conversation::talk::TalkFailure;
use serde_json::{Value, json};

pub(super) fn listed(value: &Value) -> Vec<Value> {
    match value {
        Value::Array(items) => items.clone(),
        Value::Object(_) => vec![value.clone()],
        _ => Vec::new(),
    }
}

pub(super) fn raised_or_returned(result: Value) -> Result<Option<Value>, TalkFailure> {
    match result["message"]
        .as_str()
        .filter(|_| result["raise"].is_string())
    {
        Some(message) => Err(TalkFailure::new(message)),
        None => Ok(Some(result)),
    }
}

pub(super) fn stream(ignored: bool, text: &str) -> Value {
    if ignored {
        return Value::Null;
    }
    json!(text)
}

pub(super) fn at_of(event: &Value) -> f64 {
    event["at"].as_f64().unwrap_or_default()
}

pub(super) fn sorted_events(scenario: &Value) -> Vec<Value> {
    let mut events = listed(&scenario["file_events"]);
    events.sort_by(|left, right| at_of(left).total_cmp(&at_of(right)));
    events
}
