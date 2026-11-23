use std::error::Error;

use avatar_floor::quiet::ListenerPurpose;
use avatar_routing::names::session::Session;
use avatar_wake::listener::{Setting, spoken_prompt};
use serde_json::Value;

use super::microphone::run_events;

const FIXTURE: &str = include_str!("fixtures/listener.json");

fn cases_of(fixture: &Value) -> Vec<Value> {
    fixture["cases"].as_array().cloned().unwrap_or_default()
}

#[test]
fn the_state_machine_replays_every_python_scenario() -> Result<(), Box<dyn Error>> {
    let fixture: Value = serde_json::from_str(FIXTURE)?;
    let cases = cases_of(&fixture);
    assert!(cases.len() >= 50, "the oracle covers at least 50 scenarios");
    for case in &cases {
        let events = Value::Array(run_events(&fixture["defaults"], &case["overrides"]));
        assert_eq!(events, case["events"], "scenario {}", case["name"]);
    }
    Ok(())
}

fn setting_of(purpose: &Value) -> Option<Setting> {
    match purpose.as_str() {
        None => Setting::of(None),
        Some("conversation") => Setting::of(Some(ListenerPurpose::Conversation)),
        _ => Setting::of(Some(ListenerPurpose::Meeting)),
    }
}

fn session_of(prompt: &Value) -> Session {
    Session {
        servant: prompt["servant"].as_str().unwrap_or_default().to_owned(),
        pane: prompt["pane"].as_str().unwrap_or_default().to_owned(),
        host: prompt["host"].as_str().map(str::to_owned),
    }
}

fn check(prompt: &Value) -> Result<(), Box<dyn Error>> {
    let setting = setting_of(&prompt["purpose"]).ok_or("unsupported purpose")?;
    let text = spoken_prompt("hey, go", setting, &session_of(prompt), "kira");
    assert_eq!(text, prompt["text"].as_str().unwrap_or_default());
    Ok(())
}

#[test]
fn spoken_prompts_match_the_python_text_for_every_setting() -> Result<(), Box<dyn Error>> {
    let fixture: Value = serde_json::from_str(FIXTURE)?;
    let prompts = fixture["prompts"].as_array().cloned().unwrap_or_default();
    assert_eq!(prompts.len(), 6);
    prompts.iter().try_for_each(check)
}
