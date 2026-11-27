use super::brief::{STILL_WORKING, could_not_finish};
use super::report::ReportKind;
use super::task::Verdict;
use super::{ANSWER_DEADLINE_SECONDS, STILL_WORKING_SECONDS};

#[derive(Debug, PartialEq, Eq)]
pub enum Observation {
    Answered(String),
    Failed(String),
    StillWorking,
    Waiting,
}

impl Observation {
    #[must_use]
    pub fn report(self) -> Option<(String, ReportKind)> {
        match self {
            Self::Answered(text) => Some((text, ReportKind::Answer)),
            Self::Failed(reason) => Some((could_not_finish(&reason), ReportKind::Failure(reason))),
            Self::StillWorking => Some((STILL_WORKING.to_owned(), ReportKind::Progress)),
            Self::Waiting => None,
        }
    }
}

pub struct AnswerWatch {
    deadline: f64,
    next_still_working: f64,
}

fn observation_of(verdict: Verdict) -> Observation {
    match verdict {
        Verdict::Answer(text) => Observation::Answered(text),
        Verdict::Failure(reason) => Observation::Failed(reason),
    }
}

fn observed(next_still_working: &mut f64, now: f64, verdict: Option<Verdict>) -> Observation {
    if let Some(found) = verdict {
        return observation_of(found);
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

    pub fn observe(&mut self, now: f64, verdict: Option<Verdict>) -> Observation {
        observed(&mut self.next_still_working, now, verdict)
    }
}
