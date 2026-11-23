use std::ops::ControlFlow;

use super::finished::wait_for_finished_speech;
use super::transcript::wait_since;
use super::watch::ListeningWatch;
use crate::talk::effects::CLOSED_MESSAGE;
use crate::talk::failure::TalkFailure;
use crate::talk::ports::{Clock, Files, Units};
use crate::talk::state::heard::{last_heard, remember};
use crate::utterances::{HeardEntry, consumed_utterance, still_listening};

fn listening(
    boundary: &mut (impl Units + Files + Clock),
    watch: &mut ListeningWatch,
) -> Result<bool, TalkFailure> {
    let open = watch.still_open(boundary);
    let addressed = watch.still_addressed(boundary)?;
    still_listening(open, addressed).map_err(|_| TalkFailure::new(CLOSED_MESSAGE))
}

fn first_entries_round(
    boundary: &mut (impl Units + Files + Clock),
    deadline: f64,
    watch: &mut ListeningWatch,
) -> Result<ControlFlow<Vec<HeardEntry>>, TalkFailure> {
    if !listening(boundary, watch)? {
        return Ok(ControlFlow::Break(Vec::new()));
    }
    let slice = (deadline - boundary.monotonic()).clamp(0.0, 1.0);
    let heard = last_heard(boundary)?;
    let entries = wait_since(boundary, heard, slice)?;
    let finished = !entries.is_empty() || boundary.monotonic() >= deadline;
    Ok(finished
        .then_some(entries)
        .map_or(ControlFlow::Continue(()), ControlFlow::Break))
}

fn wait_for_first_entries(
    boundary: &mut (impl Units + Files + Clock),
    wait_seconds: f64,
    watch: &mut ListeningWatch,
) -> Result<Vec<HeardEntry>, TalkFailure> {
    let deadline = boundary.monotonic() + wait_seconds;
    let mut flow = first_entries_round(boundary, deadline, watch)?;
    while flow.is_continue() {
        flow = first_entries_round(boundary, deadline, watch)?;
    }
    Ok(flow.break_value().unwrap_or_default())
}

fn consume(boundary: &mut (impl Units + Files + Clock), entries: &[HeardEntry]) -> Option<String> {
    let consumed = consumed_utterance(entries)?;
    remember(boundary, consumed.last_sequence);
    Some(consumed.text)
}

pub fn next_utterance(
    boundary: &mut (impl Units + Files + Clock),
    wait_seconds: f64,
    watch: &mut ListeningWatch,
) -> Result<Option<String>, TalkFailure> {
    let first = wait_for_first_entries(boundary, wait_seconds, watch)?;
    if first.is_empty() {
        return Ok(None);
    }
    let entries = wait_for_finished_speech(boundary, first, watch)?;
    let addressed = watch.still_addressed(boundary)?;
    Ok(addressed.then(|| consume(boundary, &entries)).flatten())
}
