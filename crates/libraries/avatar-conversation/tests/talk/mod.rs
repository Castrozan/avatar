mod arguments;
mod elsewhere;
mod entries;
mod fake;
mod properties;
mod reach;
mod scenarios;

use serde_json::Value;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn fixture_cases(fixture: &str) -> Result<Vec<Value>, serde_json::Error> {
    serde_json::from_str(fixture)
}
