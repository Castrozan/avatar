use super::boundary::Boundary;
use crate::switch::failure::LatencyError;
use crate::switch::predicate::{is_delivery, reply_from, talk_from};
use crate::switch::record::{Logged, Record};
use crate::switch::search::first_after;

pub(crate) type Pair = (Boundary, Option<f64>);

pub(crate) const LONGEST_SPEECH_SECONDS: f64 = 310.0;
const HEARD_BOUNDARIES: [Boundary; 4] = [
    Boundary::SpeechEnded,
    Boundary::SegmentClosed,
    Boundary::TranscriptionStarted,
    Boundary::TranscriptWritten,
];
const REPLY_FIELDS: [(Boundary, &str); 5] = [
    (Boundary::ReplyInvoked, "invoked"),
    (Boundary::SpeechReceived, "speech_received"),
    (Boundary::SynthesisStarted, "synthesis_started"),
    (Boundary::Synthesized, "synthesized"),
    (Boundary::FirstAudio, "playing"),
];

#[must_use]
pub(crate) fn heard_pairs(heard: Option<&Logged>) -> Vec<Pair> {
    heard.map_or_else(Vec::new, heard_entries)
}

fn heard_entries(heard: &Logged) -> Vec<Pair> {
    let mut pairs: Vec<Pair> = HEARD_BOUNDARIES
        .iter()
        .map(|boundary| (*boundary, heard.record.number(boundary.key())))
        .collect();
    pairs.push((Boundary::WakeRead, Some(heard.at)));
    pairs
}

pub(crate) fn decision_pairs(decision: Option<&Logged>) -> Result<Vec<Pair>, LatencyError> {
    decision.map_or_else(|| Ok(Vec::new()), decision_entries)
}

fn decision_entries(decision: &Logged) -> Result<Vec<Pair>, LatencyError> {
    let latency_seconds = decision.record.required_number("latency_ms")? / 1000.0;
    Ok(vec![
        (
            Boundary::DecisionStarted,
            Some(decision.at - latency_seconds),
        ),
        (Boundary::DecisionDone, Some(decision.at)),
    ])
}

pub(crate) fn delivery_pairs(anchor: &Logged) -> Result<Vec<Pair>, LatencyError> {
    if !is_delivery(&anchor.record) {
        return Ok(Vec::new());
    }
    let delivery_seconds = anchor.record.required_number("delivery_ms")? / 1000.0;
    Ok(vec![
        (Boundary::PromptSent, Some(anchor.at - delivery_seconds)),
        (Boundary::PromptDelivered, Some(anchor.at)),
    ])
}

#[must_use]
pub(crate) fn agent_pairs(logged: &[Logged], index: usize, tenure_end: f64) -> Vec<Pair> {
    let anchor = &logged[index];
    if !is_delivery(&anchor.record) {
        return Vec::new();
    }
    let pane = anchor.record.text("pane");
    let command = first_after(logged, index, tenure_end, talk_from(pane));
    let reply = reply_after(logged, index, tenure_end);
    let mut pairs = command_pairs(command);
    pairs.extend(reply_pairs(reply));
    pairs
}

fn reply_after(logged: &[Logged], index: usize, tenure_end: f64) -> Option<&Logged> {
    let anchor = &logged[index];
    let window_end = tenure_end + LONGEST_SPEECH_SECONDS;
    let pane = anchor.record.text("pane");
    first_after(
        logged,
        index,
        window_end,
        reply_from(pane, anchor.at, tenure_end),
    )
}

fn command_pairs(command: Option<&Logged>) -> Vec<Pair> {
    command
        .map(|command| (Boundary::AgentFirstCommand, Some(command.at)))
        .into_iter()
        .collect()
}

fn reply_pairs(reply: Option<&Logged>) -> Vec<Pair> {
    reply.map_or_else(Vec::new, |reply| reply_entries(&reply.record))
}

fn reply_entries(record: &Record) -> Vec<Pair> {
    REPLY_FIELDS
        .iter()
        .map(|(boundary, key)| (*boundary, record.number(key)))
        .collect()
}
