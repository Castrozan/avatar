use std::cell::RefCell;

use avatar_wake::addressing::{DecisionEndpoint, DecisionUnavailable, MODEL, answers, answers_in};
use serde_json::{Value, json};

#[derive(Default)]
struct RecordingEndpoint {
    reply: Option<Value>,
    bodies: RefCell<Vec<Value>>,
}

impl DecisionEndpoint for RecordingEndpoint {
    fn post(&self, body: &Value) -> Result<Value, DecisionUnavailable> {
        self.bodies.borrow_mut().push(body.clone());
        self.reply.clone().ok_or_else(broken_reply)
    }
}

fn broken_reply() -> DecisionUnavailable {
    DecisionUnavailable {
        reason: "The Decisions API broke its reply".to_owned(),
    }
}

#[test]
fn the_request_body_carries_the_model_input_and_questions() -> Result<(), DecisionUnavailable> {
    let endpoint = RecordingEndpoint {
        reply: Some(json!({"answers": [{"name": "addressee", "type": "choice"}]})),
        ..RecordingEndpoint::default()
    };
    let input = json!([{"role": "user", "content": "Voyager, hi"}]);
    let questions = json!([{"type": "choice", "name": "addressee"}]);
    let answered = answers(&endpoint, &input, &questions)?;
    assert_eq!(answered, json!([{"name": "addressee", "type": "choice"}]));
    assert_body(&endpoint, &input, &questions);
    Ok(())
}

fn assert_body(endpoint: &RecordingEndpoint, input: &Value, questions: &Value) {
    let bodies = endpoint.bodies.borrow();
    assert_eq!(bodies[0]["model"], json!(MODEL));
    assert_eq!(bodies[0]["input"], *input);
    assert_eq!(bodies[0]["questions"], *questions);
}

#[test]
fn a_failed_post_is_unavailable_and_never_answers() {
    let endpoint = RecordingEndpoint::default();
    let outcome = answers(&endpoint, &json!([]), &json!([]));
    assert_eq!(
        outcome.map_err(|error| error.reason),
        Err(broken_reply().reason)
    );
}

#[test]
fn a_reply_without_an_answers_list_is_unavailable() {
    for reply in [
        json!({}),
        json!({"answers": "addressee"}),
        json!([1]),
        json!(null),
    ] {
        assert!(answers_in(&reply).is_err(), "reply {reply} must be refused");
    }
}

#[test]
fn a_reply_with_an_answers_list_passes_its_list_through() {
    let reply = json!({"answers": [{"name": "addressee"}], "extra": true});
    assert_eq!(answers_in(&reply), Ok(json!([{"name": "addressee"}])));
}
