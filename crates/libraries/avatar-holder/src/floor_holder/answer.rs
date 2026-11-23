use crate::hermes_turn::{FAILED_TURN_APOLOGY, HermesTurnError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    pub report: Option<String>,
    pub reply: String,
}

#[must_use]
pub fn answer_for(reply: Result<String, HermesTurnError>) -> Answer {
    match reply {
        Ok(reply) => Answer {
            report: None,
            reply,
        },
        Err(error) => Answer {
            report: Some(error.to_string()),
            reply: FAILED_TURN_APOLOGY.to_owned(),
        },
    }
}
