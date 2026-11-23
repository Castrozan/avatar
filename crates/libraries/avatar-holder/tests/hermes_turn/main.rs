use avatar_holder::hermes_turn::HermesTurnError;
use serde_json::Value;
use std::error::Error;

mod command;
mod prompt;
mod reply;
mod turn;
mod worth;

fn cases(fixture: &str) -> Result<Vec<Value>, Box<dyn Error>> {
    Ok(serde_json::from_str(fixture)?)
}

fn case_name(case: &Value) -> &str {
    case["name"].as_str().unwrap_or_default()
}

fn outcome_matches(actual: &Result<String, HermesTurnError>, expected: &Value) -> bool {
    match (actual, expected.get("ok"), expected.get("error")) {
        (Ok(text), Some(Value::String(expected_text)), _) => text == expected_text,
        (Err(error), _, Some(Value::String(message))) => error.to_string() == *message,
        _ => false,
    }
}
