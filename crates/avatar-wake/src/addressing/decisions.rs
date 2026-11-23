use serde_json::{Value, json};

pub const MODEL: &str = "gpt-6-luna";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecisionUnavailable {
    pub reason: String,
}

pub trait DecisionEndpoint {
    fn post(&self, body: &Value) -> Result<Value, DecisionUnavailable>;
}

pub fn answers(
    endpoint: &dyn DecisionEndpoint,
    input: &Value,
    questions: &Value,
) -> Result<Value, DecisionUnavailable> {
    let body = json!({"model": MODEL, "input": input, "questions": questions});
    answers_in(&endpoint.post(&body)?)
}

pub fn answers_in(reply: &Value) -> Result<Value, DecisionUnavailable> {
    reply
        .get("answers")
        .filter(|answered| answered.is_array())
        .cloned()
        .ok_or_else(|| DecisionUnavailable {
            reason: "The Decisions API replied without a list of answers".to_owned(),
        })
}
