use super::{case_name, cases};
use crate::fakes::RecordingCommands;
use avatar_holder::holder_cli::{HolderEnd, conclude};
use serde_json::Value;
use std::error::Error;

const FIXTURE: &str = include_str!("fixtures/voice_holder_cli_exit.json");

fn end_for(scenario: &str) -> Option<HolderEnd> {
    match scenario {
        "closed" => Some(HolderEnd::Closed),
        "stopped" => Some(HolderEnd::Stopped),
        "runtime" => Some(HolderEnd::Failed("runtime boom".to_owned())),
        "os" => Some(HolderEnd::Failed("disk".to_owned())),
        "calledprocess" => Some(HolderEnd::Failed(
            "Command 'avatar-talk' returned non-zero exit status 1.".to_owned(),
        )),
        _ => None,
    }
}

fn exit_matches(outcome: &Result<(), String>, case: &Value) -> bool {
    match case["exit"].as_i64() {
        Some(0) => outcome.is_ok(),
        _ => {
            outcome.as_ref().err().map(String::as_str) == case["stderr"].as_str().map(str::trim_end)
        }
    }
}

fn conclusion_matches(case: &Value) -> bool {
    let Some(end) = end_for(case_name(case)) else {
        return false;
    };
    let mut commands = RecordingCommands::default();
    let outcome = conclude(end, &mut commands);
    let closed = commands.commands.len() == 1;
    closed == case["closed"].as_bool().unwrap_or_default() && exit_matches(&outcome, case)
}

#[test]
fn conclusion_matches_the_python_oracle() -> Result<(), Box<dyn Error>> {
    let covered = cases(FIXTURE)?
        .into_iter()
        .filter(|case| end_for(case_name(case)).is_some());
    for case in covered {
        assert!(conclusion_matches(&case), "case {}", case_name(&case));
    }
    Ok(())
}
