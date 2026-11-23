use serde_json::Value;

use crate::browser::BrowserError;

pub(super) fn settle_answer(received: Option<Value>, method: &str) -> Result<Value, BrowserError> {
    match received {
        Some(message) => into_result(&message),
        None => Err(BrowserError::NoAnswer {
            method: method.to_owned(),
        }),
    }
}

fn into_result(message: &Value) -> Result<Value, BrowserError> {
    match message.get("error").filter(|error| !error.is_null()) {
        Some(error) => Err(BrowserError::Protocol {
            message: error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
        }),
        None => Ok(message.get("result").cloned().unwrap_or(Value::Null)),
    }
}
