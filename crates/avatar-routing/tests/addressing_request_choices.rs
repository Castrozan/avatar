use avatar_routing::addressing::request::{Turn, request};
use serde_json::{Value, json};

const INSTRUCTIONS: &str = "Whom does the person speak to in their latest utterance, the last message, expecting that agent to answer or act right now? An agent counts only when the person calls it by name as the one they are talking to. Hearing a name does not make its agent the addressee: mentioning or talking about an agent, asking someone to tell, ask, contact or write to an agent, doing something for or about an agent, quoting or singing its name, or repeating the name while discussing names all leave that agent out.";
const NOBODY_DESCRIPTION: &str = "The speech is not directed at any agent: singing, thinking aloud, reading aloud, or talking to another person.";

fn candidates() -> Vec<String> {
    vec!["Ruri".to_owned(), "Hokusai".to_owned()]
}

fn holder_description(holder: &str) -> String {
    format!(
        "The person keeps talking to {holder}, the agent already in the conversation, including when other agents are only mentioned, talked about, or to be contacted through {holder}."
    )
}

fn candidate_description(name: &str) -> String {
    format!(
        "The person calls {name} by name to talk to {name} now, so {name} should answer; not when {name} is only the topic, someone to be told, asked or written to, or a name being sung or discussed."
    )
}

fn choice(value: &str, description: &str) -> Value {
    json!({"value": value, "description": description})
}

fn addressee_question(choices: &Value) -> Value {
    json!([{"type": "choice", "name": "addressee", "instructions": INSTRUCTIONS, "choices": choices}])
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
fn holder_request_choices_match_python() {
    let (_, questions) = holder_request();
    let choices = json!([
        choice("holder", &holder_description("Hokusai")),
        choice("Ruri", &candidate_description("Ruri")),
        choice("Hokusai", &candidate_description("Hokusai")),
        choice("nobody", NOBODY_DESCRIPTION)
    ]);
    assert_eq!(questions, addressee_question(&choices));
}

#[test]
fn room_request_choices_match_python() {
    let (_, questions) = request("sing it", &[], None, &["Ruri".to_owned()]);
    let choices = json!([
        choice("Ruri", &candidate_description("Ruri")),
        choice("nobody", NOBODY_DESCRIPTION)
    ]);
    assert_eq!(questions, addressee_question(&choices));
}
