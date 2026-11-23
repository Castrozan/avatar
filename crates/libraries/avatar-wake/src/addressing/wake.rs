use avatar_floor::addressee::Addressee;
use avatar_routing::addressing::decision::Verdict;
use avatar_routing::hearing::wake_phrase::{
    WakeMatch, addressed_session, opens_with_greeting, sessions_heard_anywhere,
};
use avatar_routing::names::session::Session;

use super::decide::{chosen, decided};
use super::utterance::{Utterance, by_called_name, holds_the_floor};
use super::voice::VoiceAddressing;

pub(super) fn wake_match(addressing: &VoiceAddressing, heard: &str) -> Option<WakeMatch> {
    let named = by_called_name(&live_sessions_named(addressing, heard));
    if named.is_empty() {
        return ignored(addressing, "Ignored wake: no unique live servant named");
    }
    match decided(addressing, &Utterance::wake(heard), &named) {
        None => greeted_at_the_wake(addressing, heard),
        Some(verdict) => chosen_at_the_wake(addressing, &verdict, &named, heard),
    }
}

fn chosen_at_the_wake(
    addressing: &VoiceAddressing,
    verdict: &Verdict,
    named: &[(String, Session)],
    heard: &str,
) -> Option<WakeMatch> {
    chosen(addressing, verdict, named, heard)
        .or_else(|| ignored(addressing, "Ignored wake: no live servant addressed"))
}

fn greeted_at_the_wake(addressing: &VoiceAddressing, heard: &str) -> Option<WakeMatch> {
    if !opens_with_greeting(heard) {
        return ignored(addressing, "Ignored wake: no opening greeting");
    }
    addressed(addressing, heard)
        .or_else(|| ignored(addressing, "Ignored wake: no unique live servant"))
}

pub(super) fn greeted_in_conversation(
    addressing: &VoiceAddressing,
    text: &str,
    holder_of: &dyn Fn() -> Option<Addressee>,
) -> Option<WakeMatch> {
    if !opens_with_greeting(text) {
        return None;
    }
    let found = addressed(addressing, text)?;
    if holds_the_floor(holder_of().as_ref(), &found.session) {
        return None;
    }
    Some(found)
}

pub(super) fn live_sessions_named(addressing: &VoiceAddressing, text: &str) -> Vec<Session> {
    if opens_with_greeting(text) || known_heard(addressing, text) {
        let refreshed = addressing.sessions.refreshed_sessions();
        return heard_anywhere(addressing, text, &refreshed);
    }
    Vec::new()
}

fn known_heard(addressing: &VoiceAddressing, text: &str) -> bool {
    let known = addressing.sessions.known_sessions();
    !heard_anywhere(addressing, text, &known).is_empty()
}

fn heard_anywhere(addressing: &VoiceAddressing, text: &str, sessions: &[Session]) -> Vec<Session> {
    sessions_heard_anywhere(text, sessions, addressing.vocabulary.as_ref())
}

fn addressed(addressing: &VoiceAddressing, text: &str) -> Option<WakeMatch> {
    let refreshed = addressing.sessions.refreshed_sessions();
    addressed_session(text, &refreshed, addressing.vocabulary.as_ref())
}

pub(super) fn ignored(addressing: &VoiceAddressing, text: &str) -> Option<WakeMatch> {
    addressing.boundaries.report_text(text);
    None
}
