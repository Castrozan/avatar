use std::error::Error;

use avatar_control::speech::turn::{TurnPhase, TurnStatus};

pub(crate) type TestResult = Result<(), Box<dyn Error>>;

fn unknown(kind: &str, name: &str) -> Box<dyn Error> {
    format!("unknown {kind}: {name}").into()
}

pub(crate) fn phase_named(name: &str) -> Result<TurnPhase, Box<dyn Error>> {
    match name {
        "queued" => Ok(TurnPhase::Queued),
        "synthesizing" => Ok(TurnPhase::Synthesizing),
        "playing" => Ok(TurnPhase::Playing),
        "ended" => Ok(TurnPhase::Ended),
        _ => Err(unknown("phase", name)),
    }
}

pub(crate) fn phase_or_idle(name: &str) -> Result<Option<TurnPhase>, Box<dyn Error>> {
    match name {
        "idle" => Ok(None),
        _ => phase_named(name).map(Some),
    }
}

pub(crate) fn status_named(name: &str) -> Result<TurnStatus, Box<dyn Error>> {
    match name {
        "finished" => Ok(TurnStatus::Finished),
        "interrupted" => Ok(TurnStatus::Interrupted),
        "cancelled" => Ok(TurnStatus::Cancelled),
        "failed" => Ok(TurnStatus::Failed),
        _ => Err(unknown("status", name)),
    }
}

pub(crate) fn optional_status(name: Option<&str>) -> Result<Option<TurnStatus>, Box<dyn Error>> {
    name.map(status_named).transpose()
}
