use avatar_live::translation::EventTranslator;
use serde_json::Value;

use crate::support::{Outcome, array, cases, event_json, same, text};

const FIXTURE: &str = include_str!("fixtures/translation.json");

#[derive(Default)]
struct Session {
    translator: EventTranslator,
    produced: Vec<Value>,
    replies: Vec<Value>,
}

impl Session {
    fn feed(&mut self, message: &Value) -> Outcome {
        let translation = self.translator.translate(text(message))?;
        self.produced
            .extend(translation.events.iter().map(event_json));
        self.replies.extend(translation.replies);
        Ok(())
    }
}

fn check(case: &Value) -> Outcome {
    let mut session = Session::default();
    array(&case["server"])
        .iter()
        .try_for_each(|message| session.feed(message))?;
    same(&Value::Array(session.produced), &case["events"], case);
    same(&Value::Array(session.replies), &case["replies"], case);
    Ok(())
}

#[test]
fn every_recorded_server_event_is_translated_with_its_replies() -> Outcome {
    cases(FIXTURE, 20)?.iter().try_for_each(check)
}

fn malformed(message: &str) -> bool {
    EventTranslator::default().translate(message).is_err()
}

#[test]
fn a_message_without_a_type_or_not_json_is_malformed() {
    assert!(malformed("{}"));
    assert!(malformed("nope"));
}

#[test]
fn audio_that_is_not_base64_or_has_no_event_id_is_malformed() {
    assert!(malformed(
        r#"{"type": "audio", "audio_event": {"audio_base_64": "%%%", "event_id": 1}}"#
    ));
    assert!(malformed(
        r#"{"type": "audio", "audio_event": {"audio_base_64": "AQI="}}"#
    ));
}

#[test]
fn a_ping_without_an_event_id_or_a_tool_call_without_an_id_is_malformed() {
    assert!(malformed(r#"{"type": "ping", "ping_event": {}}"#));
    assert!(malformed(
        r#"{"type": "client_tool_call", "client_tool_call": {"tool_name": "hand_off_to_agent"}}"#
    ));
}
