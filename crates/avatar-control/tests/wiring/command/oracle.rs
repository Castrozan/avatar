use avatar_control::command::{Reaction, parse_command, react, with_speech_output};
use serde_json::Value;

use crate::support::{TestResult, check};

const FIXTURE: &str = include_str!("fixtures/command_oracle.json");

fn section(name: &str) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let fixture: Value = serde_json::from_str(FIXTURE)?;
    Ok(fixture[name].as_array().cloned().unwrap_or_default())
}

fn arguments_of(case: &Value) -> Option<Vec<String>> {
    case["args"]
        .as_array()?
        .iter()
        .map(|item| item.as_str().map(str::to_owned))
        .collect()
}

fn check_parse(case: &Value) -> TestResult {
    let Some(arguments) = arguments_of(case) else {
        return Ok(());
    };
    let parsed = parse_command(&arguments);
    let expected_error = case["error"].as_str();
    check(parsed.as_ref().err().copied() == expected_error, case)?;
    check(
        parsed.ok().as_ref() == case.get("command").filter(|command| !command.is_null()),
        case,
    )
}

#[test]
fn commands_are_built_exactly_like_the_node_builders() -> TestResult {
    section("parse")?.iter().try_for_each(check_parse)
}

fn check_output(case: &Value) -> TestResult {
    let seat = case["seat"].as_bool().unwrap_or_default();
    let resolved = with_speech_output(case["command"].clone(), seat);
    check(resolved == case["result"], case)
}

#[test]
fn speech_output_defaults_follow_the_seat() -> TestResult {
    section("withSpeechOutput")?
        .iter()
        .try_for_each(check_output)
}

fn expected_reaction(case: &Value) -> Reaction {
    if !case["sent"].as_array().is_some_and(Vec::is_empty) {
        return Reaction::SendCommand;
    }
    match case["closed"].as_bool() {
        Some(true) => Reaction::Finish {
            printed: case["logs"][0].as_str().unwrap_or_default().to_owned(),
            failed: case["exit"] == 1,
        },
        _ => Reaction::Ignore,
    }
}

fn check_reaction(case: &Value) -> TestResult {
    let command = serde_json::json!({"id": "abc", "type": "x"});
    let reaction = react(&case["response"], &command);
    let expected = expected_reaction(case);
    check(reaction_matches(&reaction, &expected), case)
}

fn reaction_matches(found: &Reaction, expected: &Reaction) -> bool {
    match (found, expected) {
        (
            Reaction::Finish { printed, failed },
            Reaction::Finish {
                printed: wanted,
                failed: wanted_failure,
            },
        ) => failed == wanted_failure && same_json(printed, wanted),
        _ => found == expected,
    }
}

fn same_json(left: &str, right: &str) -> bool {
    serde_json::from_str::<Value>(left).ok() == serde_json::from_str::<Value>(right).ok()
}

#[test]
fn responses_are_handled_like_the_node_message_handler() -> TestResult {
    section("reactions")?.iter().try_for_each(check_reaction)
}
