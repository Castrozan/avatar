use super::{ANSWER_DEADLINE_SECONDS, STILL_WORKING_SECONDS};

#[derive(Debug, PartialEq, Eq)]
pub enum Observation {
    Answered(String),
    StillWorking,
    Waiting,
}

pub struct AnswerWatch {
    deadline: f64,
    next_still_working: f64,
}

fn observed(next_still_working: &mut f64, now: f64, answer: Option<String>) -> Observation {
    if let Some(text) = answer {
        return Observation::Answered(text);
    }
    if now < *next_still_working {
        return Observation::Waiting;
    }
    *next_still_working += STILL_WORKING_SECONDS;
    Observation::StillWorking
}

impl AnswerWatch {
    #[must_use]
    pub fn new(now: f64) -> Self {
        Self {
            deadline: now + ANSWER_DEADLINE_SECONDS,
            next_still_working: now + STILL_WORKING_SECONDS,
        }
    }

    #[must_use]
    pub fn is_open(&self, now: f64) -> bool {
        now < self.deadline
    }

    pub fn observe(&mut self, now: f64, answer: Option<String>) -> Observation {
        observed(&mut self.next_still_working, now, answer)
    }
}
