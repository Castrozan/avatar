use serde::Deserialize;
use serde_json::{Value, json};

use crate::addressing::decision::verdict;
use crate::addressing::request::{Turn, request};

#[derive(Deserialize)]
pub struct TurnRequest {
    pub speaker: Option<String>,
    pub text: String,
}

impl From<&TurnRequest> for Turn {
    fn from(turn: &TurnRequest) -> Self {
        Self {
            speaker: turn.speaker.clone(),
            text: turn.text.clone(),
        }
    }
}

#[derive(Deserialize)]
pub struct DecisionPromptRequest {
    pub utterance: String,
    pub turns: Vec<TurnRequest>,
    pub holder: Option<String>,
    pub candidates: Vec<String>,
}

impl DecisionPromptRequest {
    #[must_use]
    pub fn answer(&self) -> Value {
        let turns: Vec<Turn> = self.turns.iter().map(Turn::from).collect();
        let (messages, questions) = request(
            &self.utterance,
            &turns,
            self.holder.as_deref(),
            &self.candidates,
        );
        json!({"messages": messages, "questions": questions})
    }
}

#[derive(Deserialize)]
pub struct DecisionVerdictRequest {
    pub answers: Value,
    pub candidates: Vec<String>,
}

impl DecisionVerdictRequest {
    #[must_use]
    pub fn answer(&self) -> Value {
        let decided = verdict(&self.answers, &self.candidates);
        json!({"route": decided.route, "choice": decided.choice, "confidence": decided.confidence})
    }
}
