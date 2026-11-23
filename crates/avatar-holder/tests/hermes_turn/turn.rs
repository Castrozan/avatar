use super::{case_name, cases, outcome_matches};
use avatar_holder::hermes_turn::{
    TURN_TIMEOUT_SECONDS, TurnProcess, TurnRequest, TurnRunner, reply_to,
};
use proptest::prelude::*;
use serde_json::Value;
use std::error::Error;

const FIXTURE: &str = include_str!("fixtures/hermes_turn_reply.json");
const RESULT_LINE: &str = r#"{"type":"result","text":"Oi."}"#;
const REQUEST: TurnRequest = TurnRequest {
    profile: "silver",
    heard: "oi",
    day: "2026-11-23",
};

#[derive(Default)]
struct ScriptedTurn {
    process: Option<TurnProcess>,
    timeouts: Vec<u64>,
    commands: Vec<Vec<String>>,
}

impl ScriptedTurn {
    fn answering(process: TurnProcess) -> Self {
        Self {
            process: Some(process),
            ..Self::default()
        }
    }
}

impl TurnRunner for ScriptedTurn {
    fn run_turn(&mut self, command: &[String], timeout_seconds: u64) -> TurnProcess {
        self.timeouts.push(timeout_seconds);
        self.commands.push(command.to_vec());
        self.process
            .clone()
            .unwrap_or_else(|| TurnProcess::CouldNotRun(String::new()))
    }
}

fn finished(exit_code: i32, stdout: &str) -> TurnProcess {
    TurnProcess::Finished {
        exit_code,
        stdout: stdout.to_owned(),
    }
}

fn process_for_scenario(scenario: &str) -> TurnProcess {
    match scenario {
        "exit three" => finished(3, RESULT_LINE),
        "timeout" => {
            TurnProcess::CouldNotRun("Command 'hermes' timed out after 120 seconds".into())
        }
        "missing binary" => TurnProcess::CouldNotRun("hermes".into()),
        "empty stdout" => finished(0, ""),
        "nonzero with result" => finished(1, RESULT_LINE),
        _ => finished(0, RESULT_LINE),
    }
}

fn reply_matches(case: &Value) -> bool {
    let mut runner = ScriptedTurn::answering(process_for_scenario(case_name(case)));
    let actual = reply_to(&mut runner, &REQUEST);
    outcome_matches(&actual, &case["expected"]) && runner.timeouts == [TURN_TIMEOUT_SECONDS]
}

#[test]
fn reply_to_matches_the_python_oracle() -> Result<(), Box<dyn Error>> {
    for case in cases(FIXTURE)? {
        assert!(reply_matches(&case), "case {}", case_name(&case));
    }
    Ok(())
}

#[test]
fn the_turn_runs_the_profile_in_the_daily_voice_session() {
    let mut runner = ScriptedTurn::answering(finished(0, RESULT_LINE));
    assert_eq!(reply_to(&mut runner, &REQUEST).ok(), Some("Oi.".to_owned()));
    let command = &runner.commands[0];
    assert_eq!(&command[..5], ["hermes", "-p", "silver", "chat", "--query"]);
    assert_eq!(command[8], "voice-call-2026-11-23");
}

proptest! {
    #[test]
    fn any_failing_exit_status_is_reported_with_its_code(exit_code in 1i32..=255) {
        let mut runner = ScriptedTurn::answering(finished(exit_code, RESULT_LINE));
        let error = reply_to(&mut runner, &REQUEST).err().map(|error| error.to_string());
        prop_assert_eq!(error, Some(format!("The Hermes turn exited {exit_code}")));
    }
}
