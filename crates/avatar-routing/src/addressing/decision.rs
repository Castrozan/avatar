use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::addressing::request::{NOBODY, QUESTION};

pub const CONFIDENCE_FLOOR: f64 = 0.8;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Route {
    Candidate,
    Keep,
    Nobody,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Verdict {
    pub route: Route,
    pub choice: Option<String>,
    pub confidence: Option<f64>,
}

#[must_use]
pub fn verdict(answers: &Value, candidates: &[String]) -> Verdict {
    let answer = addressee_answer(answers);
    let choice = answer.and_then(chosen_value);
    let confidence = answer.and_then(confidence_of);
    let route = route_of(choice.as_deref(), confidence, candidates);
    Verdict {
        route,
        choice,
        confidence,
    }
}

fn addressee_answer(answers: &Value) -> Option<&Value> {
    answers
        .as_array()?
        .iter()
        .find(|answer| answer.get("name") == Some(&json!(QUESTION)))
}

fn chosen_value(answer: &Value) -> Option<String> {
    let is_choice = answer.get("type") == Some(&json!("choice"));
    is_choice
        .then(|| answer.get("choice")?.as_str().map(str::to_owned))
        .flatten()
}

fn confidence_of(answer: &Value) -> Option<f64> {
    answer
        .get("confidence")?
        .as_f64()
        .filter(|confidence| (0.0..=1.0).contains(confidence))
}

fn route_of(choice: Option<&str>, confidence: Option<f64>, candidates: &[String]) -> Route {
    match choice {
        Some(NOBODY) => Route::Nobody,
        Some(name) if is_candidate(name, confidence, candidates) => Route::Candidate,
        _ => Route::Keep,
    }
}

fn is_candidate(name: &str, confidence: Option<f64>, candidates: &[String]) -> bool {
    confidence.is_some_and(|value| value >= CONFIDENCE_FLOOR)
        && candidates.iter().any(|candidate| candidate == name)
}
