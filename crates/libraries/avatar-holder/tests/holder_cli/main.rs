use serde_json::Value;
use std::error::Error;

mod argv;
mod exit;
mod fakes;
mod hold;
mod parse;
mod properties;

fn cases(fixture: &str) -> Result<Vec<Value>, Box<dyn Error>> {
    Ok(serde_json::from_str(fixture)?)
}

fn case_name(case: &Value) -> &str {
    case["name"].as_str().unwrap_or_default()
}
