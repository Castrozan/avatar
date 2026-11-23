use avatar_floor::addressee::Addressee;
use avatar_routing::addressing::decision::{Route, Verdict};
use avatar_routing::addressing::request::request;
use avatar_routing::hearing::wake_phrase::WakeMatch;
use avatar_routing::names::session::Session;
use avatar_routing::names::spoken_names::called_name;
use serde_json::Value;

use super::report::DecisionReport;
use super::utterance::Utterance;
use super::voice::VoiceAddressing;

const MESSAGE_EDGE: &[char] = &[' ', ',', '.', ';', ':', '!', '?', '-'];

pub(super) fn decided(
    addressing: &VoiceAddressing,
    utterance: &Utterance,
    candidates: &[(String, Session)],
) -> Option<Verdict> {
    let names: Vec<String> = candidates.iter().map(|(name, _)| name.clone()).collect();
    let (input, questions) = request_for(utterance, &names);
    let report = DecisionReport::begin(addressing.boundaries, utterance, &names);
    match addressing.boundaries.decide(&input, &questions) {
        Ok(answers) => Some(report.answered(&answers)),
        Err(unavailable) => {
            report.unavailable(&unavailable);
            None
        }
    }
}

fn request_for(utterance: &Utterance, names: &[String]) -> (Value, Value) {
    let holder = utterance.holder.as_ref().and_then(holder_name);
    request(
        &utterance.text,
        &utterance.context,
        holder.as_deref(),
        names,
    )
}

pub(super) fn chosen(
    addressing: &VoiceAddressing,
    verdict: &Verdict,
    candidates: &[(String, Session)],
    text: &str,
) -> Option<WakeMatch> {
    let session = chosen_session(verdict, candidates)?;
    if addressing.sessions.refreshed_sessions().contains(&session) {
        let message = text.trim_matches(MESSAGE_EDGE).to_owned();
        return Some(WakeMatch { session, message });
    }
    addressing
        .boundaries
        .report_text(&format!("Kept the route: {} left", session.servant));
    None
}

fn chosen_session(verdict: &Verdict, candidates: &[(String, Session)]) -> Option<Session> {
    if verdict.route != Route::Candidate {
        return None;
    }
    let choice = verdict.choice.as_deref()?;
    candidates
        .iter()
        .find(|(name, _)| name == choice)
        .map(|(_, session)| session.clone())
}

fn holder_name(held: &Addressee) -> Option<String> {
    called_name(&held.servant)
}
