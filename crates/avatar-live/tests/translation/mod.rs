use avatar_live::translation::EventTranslator;
use serde_json::Value;

use crate::support::{Outcome, array, cases, event_json, same, text};

const FIXTURE: &str = include_str!("fixtures/translation.json");

#[derive(Default)]
struct Session {
    translator: EventTranslator,
    produced: Vec<Value>,
    closed: bool,
}

impl Session {
    fn accept(&mut self, message: &Value) -> Outcome {
        let translation = self.translator.translate(text(message))?;
        let events = translation.events.iter().map(event_json);
        self.produced.extend(events.collect::<Result<Vec<_>, _>>()?);
        self.closed = translation.closed;
        Ok(())
    }

    fn feed(&mut self, message: &Value) -> Outcome {
        Some(message)
            .filter(|_| !self.closed)
            .map_or(Ok(()), |open| self.accept(open))
    }
}

fn check(case: &Value) -> Outcome {
    let mut session = Session::default();
    array(&case["server"])
        .iter()
        .try_for_each(|message| session.feed(message))?;
    same(&Value::Array(session.produced), &case["events"], case);
    Ok(())
}

#[test]
fn every_recorded_python_translation_is_reproduced() -> Outcome {
    cases(FIXTURE, 20)?.iter().try_for_each(check)
}

#[test]
fn a_message_without_a_type_is_malformed() {
    assert!(EventTranslator::default().translate("{}").is_err());
}

#[test]
fn a_message_that_is_not_json_is_malformed() {
    assert!(EventTranslator::default().translate("nope").is_err());
}

#[test]
fn a_delta_that_is_not_base64_is_malformed() {
    let message = r#"{"type": "session.output_audio.delta", "delta": "%%%"}"#;
    assert!(EventTranslator::default().translate(message).is_err());
}
