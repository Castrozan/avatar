use super::{case_name, cases};
use crate::fakes::{RecordingCommands, ScriptedSession};
use avatar_holder::default_holder::DefaultHolder;
use avatar_holder::holder_cli::{CONVERSATION_NOT_OURS, HolderEnd, hold_the_conversation};
use serde_json::Value;
use std::error::Error;

const FIXTURE: &str = include_str!("fixtures/voice_holder_cli_hold.json");

fn session_for(name: &str) -> ScriptedSession {
    match name {
        "configured is ours" => ScriptedSession::configured_as(Some(silver())),
        "configured other holder" => {
            ScriptedSession::configured_as(Some(DefaultHolder::new("ana", None)))
        }
        _ => ScriptedSession::configured_as(None),
    }
}

fn silver() -> DefaultHolder {
    DefaultHolder::new("silver", Some("darkness-shibu".to_owned()))
}

fn hold_matches(case: &Value) -> bool {
    let mut commands = RecordingCommands::default();
    let mut session = session_for(case_name(case));
    let end = hold_the_conversation(&mut commands, &mut session, &silver());
    let expected_end = match case["expected"].get("error") {
        Some(_) => HolderEnd::Failed(CONVERSATION_NOT_OURS.to_owned()),
        None => HolderEnd::Closed,
    };
    end == expected_end && (session.held == 1) == case["held"].as_bool().unwrap_or_default()
}

#[test]
fn holding_matches_the_python_oracle() -> Result<(), Box<dyn Error>> {
    for case in cases(FIXTURE)? {
        assert!(hold_matches(&case), "case {}", case_name(&case));
    }
    Ok(())
}

#[test]
fn a_refused_opening_ends_without_holding() {
    let refusal = "Command 'avatar-talk' returned non-zero exit status 1.";
    let mut commands = RecordingCommands {
        failure: Some(refusal.to_owned()),
        ..RecordingCommands::default()
    };
    let mut session = ScriptedSession::configured_as(Some(silver()));
    let end = hold_the_conversation(&mut commands, &mut session, &silver());
    assert_eq!(end, HolderEnd::Failed(refusal.to_owned()));
    assert_eq!(session.held, 0);
}
