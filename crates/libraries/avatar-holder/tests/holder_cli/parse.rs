use super::cases;
use avatar_holder::holder_cli::parse_arguments;
use serde_json::{Value, json};
use std::error::Error;

const FIXTURE: &str = include_str!("fixtures/voice_holder_cli_parse.json");

fn parse_matches(case: &Value) -> bool {
    let arguments: Vec<String> = serde_json::from_value(case["argv"].clone()).unwrap_or_default();
    match parse_arguments(&arguments) {
        Ok(holder) => {
            case["expected"]["ok"]
                == json!({"hermes_profile": holder.hermes_profile, "character": holder.character})
        }
        Err(_) => case["expected"]["error"] == Value::Bool(true),
    }
}

#[test]
fn parse_arguments_matches_the_python_oracle() -> Result<(), Box<dyn Error>> {
    for case in cases(FIXTURE)? {
        assert!(parse_matches(&case), "argv {}", case["argv"]);
    }
    Ok(())
}
