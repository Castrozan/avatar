use std::cell::RefCell;
use std::error::Error;

use avatar_routing::hearing::wake_phrase::WakeMatch;
use avatar_routing::names::session::Session;
use avatar_wake::addressing::VoiceAddressing;
use serde_json::{Value, json};

use super::fakes::FakeAddressing;
use super::values::{
    addressee_of, answers_of, entry_of, floats, seat_of, sessions_at, sessions_or,
};

const FIXTURE: &str = include_str!("fixtures/voice_addressing.json");

#[test]
fn voice_addressing_replays_every_python_case() -> Result<(), Box<dyn Error>> {
    let fixture: Value = serde_json::from_str(FIXTURE)?;
    let default_sessions = sessions_at(&fixture["default_sessions"]);
    let cases = fixture["cases"].as_array().cloned().unwrap_or_default();
    assert!(cases.len() >= 40, "the oracle covers at least 40 cases");
    for case in &cases {
        check_case(case, &default_sessions);
    }
    Ok(())
}

fn check_case(case: &Value, default_sessions: &[Session]) {
    assert_eq!(
        run_case(case, default_sessions),
        case["expected"],
        "case {}",
        case["name"]
    );
}

fn run_case(case: &Value, default_sessions: &[Session]) -> Value {
    let fake = fake_for(case, default_sessions);
    let results = run_ops(&fake, case);
    json!({
        "results": results,
        "reports": fake.reports.borrow().clone(),
        "decide_calls": fake.decide_calls.borrow().iter().map(summarize).collect::<Vec<_>>(),
    })
}

fn run_ops(fake: &FakeAddressing, case: &Value) -> Vec<Value> {
    let mut addressing = VoiceAddressing::new(fake, fake, None);
    case["ops"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|op| run_op(&mut addressing, op))
        .collect()
}

fn run_op(addressing: &mut VoiceAddressing, op: &Value) -> Value {
    let found = if op["op"] == "wake" {
        addressing.wake_match(&text_of(&op["text"]))
    } else {
        addressing.conversation_match(&entry_of(&op["entry"]))
    };
    found.map_or(Value::Null, |wake| match_json(&wake))
}

fn match_json(wake: &WakeMatch) -> Value {
    json!({
        "servant": wake.session.servant,
        "pane": wake.session.pane,
        "host": wake.session.host,
        "message": wake.message,
    })
}

fn text_of(value: &Value) -> String {
    value.as_str().unwrap_or_default().to_owned()
}

fn fake_for(case: &Value, default_sessions: &[Session]) -> FakeAddressing {
    FakeAddressing {
        holder: addressee_of(&case["holder"]),
        seat: seat_of(&case["seat"]),
        clock: floats(&case["clock"]),
        answers: RefCell::new(answers_of(&case["answers"]).into()),
        known: sessions_or(&case["known"], default_sessions),
        refreshed: sessions_or(&case["refreshed"], default_sessions),
        ..FakeAddressing::default()
    }
}

fn summarize(call: &(Value, Value)) -> Value {
    let (input, questions) = call;
    let messages = input.as_array().cloned().unwrap_or_default();
    json!({
        "choices": choice_values(questions),
        "message_count": messages.len(),
        "utterance": messages.last().map_or(Value::Null, |last| last["content"].clone()),
    })
}

fn choice_values(questions: &Value) -> Vec<Value> {
    questions[0]["choices"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|choice| choice["value"].clone())
        .collect()
}
