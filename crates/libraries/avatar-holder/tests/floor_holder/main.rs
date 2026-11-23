use serde_json::Value;
use std::error::Error;

mod answer;
mod builders;
mod holding;
mod observe;
mod room;
mod throttle;
mod voice;

fn cases(fixture: &str) -> Result<Vec<Value>, Box<dyn Error>> {
    Ok(serde_json::from_str(fixture)?)
}
