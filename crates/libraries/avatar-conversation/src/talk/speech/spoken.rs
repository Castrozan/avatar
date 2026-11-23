use serde_json::{Map, Value, json};

use super::SPEECH_TIMEOUT_SECONDS;
use super::turn::{remember_stopped_reply, withheld_reason, withhold};
use crate::talk::failure::TalkFailure;
use crate::talk::ports::exec::{check_status, run_unchecked};
use crate::talk::ports::{ActivityLog, Clock, Environment, Files, Subprocess};
use crate::talk::state::this_pane;
use crate::turn_taking::{WITHHELD_STATUS, WithheldReason, outcome_stops_reply};
use crate::voice_turns::{acknowledgement, speech_moments};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletedSpeech {
    pub argv: Vec<String>,
    pub status: i32,
    pub stdout: String,
}

impl CompletedSpeech {
    fn withheld(argv: Vec<String>, line: &str) -> Self {
        Self {
            argv,
            status: 0,
            stdout: format!("{line}\n"),
        }
    }

    pub fn check(&self) -> Result<(), TalkFailure> {
        check_status(&self.argv, self.status)
    }

    #[must_use]
    pub fn status_text(&self) -> Option<String> {
        let response = acknowledgement(&self.stdout)?;
        response.get("status")?.as_str().map(str::to_owned)
    }
}

fn details_of<'key>(pairs: impl IntoIterator<Item = (&'key str, Value)>) -> Map<String, Value> {
    pairs
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value))
        .collect()
}

fn field(response: &Value, key: &str) -> Value {
    response.get(key).cloned().unwrap_or(Value::Null)
}

fn speech_details(invoked: f64, response: &Value) -> Map<String, Value> {
    let fixed = [
        ("invoked", json!(invoked)),
        ("status", field(response, "status")),
        ("source", field(response, "source")),
    ];
    let moments = speech_moments(response)
        .into_iter()
        .map(|(moment, seconds)| (moment.name(), json!(seconds)));
    details_of(fixed.into_iter().chain(moments))
}

fn report_spoken(boundary: &mut (impl ActivityLog + Environment), invoked: f64, stdout: &str) {
    let response = acknowledgement(stdout).unwrap_or_else(|| json!({}));
    let pane = this_pane(boundary);
    boundary.report_speech(pane.as_deref(), &speech_details(invoked, &response));
}

pub(super) fn spoken(
    boundary: &mut (impl Subprocess + ActivityLog + Environment),
    argv: Vec<String>,
    invoked: f64,
) -> Result<CompletedSpeech, TalkFailure> {
    let output = run_unchecked(boundary, &argv, Some(SPEECH_TIMEOUT_SECONDS))?;
    report_spoken(boundary, invoked, &output.stdout);
    Ok(CompletedSpeech {
        argv,
        status: output.status,
        stdout: output.stdout,
    })
}

fn withheld_speech(
    boundary: &mut (impl Files + ActivityLog + Environment),
    argv: Vec<String>,
    reason: WithheldReason,
    invoked: f64,
) -> CompletedSpeech {
    let line = withhold(boundary, reason);
    let details = details_of([
        ("invoked", json!(invoked)),
        ("status", json!(WITHHELD_STATUS)),
        ("reason", json!(reason.as_str())),
    ]);
    boundary.report_speech(this_pane(boundary).as_deref(), &details);
    CompletedSpeech::withheld(argv, &line)
}

pub fn spoken_in_turn(
    boundary: &mut (impl Subprocess + ActivityLog + Environment + Files + Clock),
    argv: Vec<String>,
) -> Result<CompletedSpeech, TalkFailure> {
    let invoked = boundary.wall();
    if let Some(reason) = withheld_reason(boundary)? {
        return Ok(withheld_speech(boundary, argv, reason, invoked));
    }
    let completed = spoken(boundary, argv, invoked)?;
    if outcome_stops_reply(completed.status_text().as_deref()) {
        remember_stopped_reply(boundary);
    }
    Ok(completed)
}
