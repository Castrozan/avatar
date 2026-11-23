use std::collections::BTreeMap;

use avatar_listening::conversation_state::{ListenerProbe, ListenerPurpose, live_purpose};
use serde_json::Value;

use crate::support::{Outcome, fixture, rows};

const FIXTURE: &str = "conversation_state/fixtures/live_purpose.json";

#[derive(Default)]
struct ScriptedListener {
    active: BTreeMap<String, bool>,
    configured: Option<String>,
}

impl ListenerProbe for ScriptedListener {
    fn unit_active(&mut self, unit: &str) -> bool {
        self.active.get(unit).copied().unwrap_or(false)
    }

    fn configured_purpose(&mut self) -> Option<String> {
        self.configured.clone()
    }
}

fn listener_of(case: &Value) -> Result<ScriptedListener, Box<dyn std::error::Error>> {
    let mut listener = ScriptedListener::default();
    for (unit, active) in case["active"].as_object().ok_or("active")? {
        let flag = active.as_bool().ok_or("active flag")?;
        listener.active.insert(unit.clone(), flag);
    }
    listener.configured = case["configuration"]["purpose"].as_str().map(str::to_owned);
    Ok(listener)
}

fn purpose_name(purpose: Option<ListenerPurpose>) -> Option<&'static str> {
    match purpose {
        None => None,
        Some(ListenerPurpose::Conversation) => Some("conversation"),
        Some(ListenerPurpose::Meeting) => Some("meeting"),
        Some(ListenerPurpose::Live) => Some("live"),
    }
}

fn replay(case: &Value) -> Outcome {
    let mut listener = listener_of(case)?;
    let expected = case["purpose"].as_str();
    assert_eq!(
        purpose_name(live_purpose(&mut listener)),
        expected,
        "{case}"
    );
    Ok(())
}

#[test]
fn every_oracle_listener_purpose_matches_the_python_session() -> Outcome {
    let document = fixture(FIXTURE)?;
    let cases = rows(&document["cases"])?;
    assert_eq!(cases.len(), 12);
    cases.iter().try_for_each(replay)
}

#[test]
fn a_purpose_is_only_read_while_something_captures() {
    let mut listener = ScriptedListener {
        active: BTreeMap::new(),
        configured: Some("live".to_owned()),
    };
    assert_eq!(live_purpose(&mut listener), None);
}
