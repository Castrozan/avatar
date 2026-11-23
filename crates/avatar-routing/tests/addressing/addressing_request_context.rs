use avatar_routing::addressing::request::{Turn, request};
use serde_json::{Value, json};

const SPEECH: &str = "Every utterance is a speech-to-text transcript of the person, so names and words may be misheard.";

fn candidates() -> Vec<String> {
    vec!["Ruri".to_owned(), "Hokusai".to_owned()]
}

fn user(content: &str) -> Value {
    json!({"role": "user", "content": content})
}

fn holder_turns() -> [Turn; 2] {
    [
        Turn {
            speaker: Some("Ana".to_owned()),
            text: "hi there".to_owned(),
        },
        Turn {
            speaker: None,
            text: "ok".to_owned(),
        },
    ]
}

fn holder_request() -> (Vec<Value>, Value) {
    let (messages, questions) = request(
        "Ruri, are you there?",
        &holder_turns(),
        Some("Hokusai"),
        &candidates(),
    );
    (messages.as_array().cloned().unwrap_or_default(), questions)
}

#[test]
fn holder_request_developer_context_matches_python() {
    let (messages, _) = holder_request();
    let developer = format!(
        "A person is talking by voice with an AI agent named Hokusai. Other live AI agents named Ruri, Hokusai can take over the conversation when the person turns to address them. {SPEECH}"
    );
    assert_eq!(
        messages[0],
        json!({"role": "developer", "content": developer})
    );
}

#[test]
fn holder_request_turns_match_python() {
    let (messages, _) = holder_request();
    assert_eq!(
        messages[1..],
        [
            user("(earlier) Ana: hi there"),
            user("(earlier) ok"),
            user("Ruri, are you there?")
        ]
    );
}

#[test]
fn room_request_developer_context_matches_python() {
    let (messages, _) = request("sing it", &[], None, &["Ruri".to_owned()]);
    let developer = format!(
        "A room microphone hears a person near voice-activated AI agents named Ruri. Nobody is in a conversation with them yet; an agent wakes only when the person calls it to speak with it. {SPEECH}"
    );
    assert_eq!(
        messages,
        json!([{"role": "developer", "content": developer}, user("sing it")])
    );
}

#[test]
fn empty_speaker_is_treated_as_absent() {
    let turns = [Turn {
        speaker: Some(String::new()),
        text: "x".to_owned(),
    }];
    let (messages, _) = request("x", &turns, None, &["Ruri".to_owned()]);
    assert_eq!(messages[1], user("(earlier) x"));
}
