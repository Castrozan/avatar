use crate::addressing::texts::{
    NOBODY_DESCRIPTION, QUESTION_INSTRUCTIONS, candidate_description, developer_context,
    holder_description,
};
use serde_json::{Value, json};

pub const QUESTION: &str = "addressee";
pub const HOLDER: &str = "holder";
pub const NOBODY: &str = "nobody";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Turn {
    pub speaker: Option<String>,
    pub text: String,
}

#[must_use]
pub fn request(
    utterance: &str,
    turns: &[Turn],
    holder: Option<&str>,
    candidates: &[String],
) -> (Value, Value) {
    let listed_candidates = candidates.join(", ");
    let mut messages = vec![
        json!({"role": "developer", "content": developer_context(holder, &listed_candidates)}),
    ];
    messages.extend(turns.iter().map(turn_message));
    messages.push(json!({"role": "user", "content": utterance}));
    (
        Value::Array(messages),
        Value::Array(vec![question(holder, candidates)]),
    )
}

fn turn_message(turn: &Turn) -> Value {
    let said = match turn.speaker.as_deref() {
        Some(speaker) if !speaker.is_empty() => format!("{speaker}: {}", turn.text),
        _ => turn.text.clone(),
    };
    json!({"role": "user", "content": format!("(earlier) {said}")})
}

fn question(holder: Option<&str>, candidates: &[String]) -> Value {
    json!({
        "type": "choice",
        "name": QUESTION,
        "instructions": QUESTION_INSTRUCTIONS,
        "choices": choices(holder, candidates),
    })
}

fn choices(holder: Option<&str>, candidates: &[String]) -> Vec<Value> {
    holder
        .map(holder_choice)
        .into_iter()
        .chain(
            candidates
                .iter()
                .map(|candidate| candidate_choice(candidate)),
        )
        .chain(std::iter::once(nobody_choice()))
        .collect()
}

fn holder_choice(holder: &str) -> Value {
    json!({"value": HOLDER, "description": holder_description(holder)})
}

fn candidate_choice(name: &str) -> Value {
    json!({"value": name, "description": candidate_description(name)})
}

fn nobody_choice() -> Value {
    json!({"value": NOBODY, "description": NOBODY_DESCRIPTION})
}
