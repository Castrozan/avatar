use avatar_control::site::http_date;
use serde_json::Value;

use crate::support::{TestResult, check};

#[test]
fn http_dates_match_the_python_email_formatter() -> TestResult {
    let cases: Vec<Value> = serde_json::from_str(include_str!("fixtures/dates.json"))?;
    for case in &cases {
        let seconds = case["seconds"].as_i64().unwrap_or_default();
        check(
            Some(http_date(seconds).as_str()) == case["text"].as_str(),
            case,
        )?;
    }
    Ok(())
}
