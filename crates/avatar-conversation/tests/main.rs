mod conversation_report;
mod room_reach;
mod turn_taking;
mod utterances;
mod voice_turns;

use serde_json::Value;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn replay(fixture: &str, function: &str, check: impl Fn(&Value) -> TestResult) -> TestResult {
    let cases: Vec<Value> = serde_json::from_str(fixture)?;
    let selected: Vec<&Value> = cases
        .iter()
        .filter(|case| case["function"] == function)
        .collect();
    assert!(!selected.is_empty(), "no {function} cases");
    selected.into_iter().try_for_each(check)
}

fn expected_bool(case: &Value) -> Result<bool, Box<dyn std::error::Error>> {
    case["expected"]
        .as_bool()
        .ok_or_else(|| format!("expected is not a bool: {case}").into())
}
