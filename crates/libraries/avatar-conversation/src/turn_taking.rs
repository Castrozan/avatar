use serde_json::{Value, json};

pub const ACKNOWLEDGEMENT_TYPE: &str = "speakAck";
pub const REACTION_SECONDS: f64 = 1.0;
const STOPPED_STATUSES: [&str; 2] = ["interrupted", "cancelled"];
pub const WITHHELD_STATUS: &str = "withheld";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WithheldReason {
    Queued,
    Speaking,
    Unheard,
}

impl WithheldReason {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Speaking => "speaking",
            Self::Unheard => "unheard",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TurnObservation {
    pub queued_behind_stopped_reply: bool,
    pub speech_in_progress: bool,
    pub unheard_words: bool,
}

#[must_use]
pub fn withheld_reason(observation: TurnObservation) -> Option<WithheldReason> {
    if observation.queued_behind_stopped_reply {
        return Some(WithheldReason::Queued);
    }
    if observation.speech_in_progress {
        return Some(WithheldReason::Speaking);
    }
    observation.unheard_words.then_some(WithheldReason::Unheard)
}

#[must_use]
pub fn queued_behind_a_stopped_reply(stopped_reply_age_seconds: Option<f64>) -> bool {
    stopped_reply_age_seconds.is_some_and(|age| age < REACTION_SECONDS)
}

#[must_use]
pub fn outcome_stops_reply(status: Option<&str>) -> bool {
    status.is_some_and(|value| STOPPED_STATUSES.contains(&value))
}

#[must_use]
pub fn withhold_acknowledgement(reason: WithheldReason) -> Value {
    json!({"type": ACKNOWLEDGEMENT_TYPE, "status": WITHHELD_STATUS, "reason": reason.as_str()})
}
