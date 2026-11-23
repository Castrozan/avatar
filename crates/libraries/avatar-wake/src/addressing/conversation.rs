use avatar_floor::addressee::Addressee;
use avatar_floor::quiet::Seat;
use avatar_routing::addressing::request::Turn;
use avatar_routing::hearing::wake_phrase::WakeMatch;
use avatar_routing::names::session::Session;

use super::decide::{chosen, decided};
use super::utterance::{ConversationEntry, Phase, Utterance, by_called_name, holds_the_floor};
use super::voice::VoiceAddressing;
use super::wake::{greeted_in_conversation, live_sessions_named};

const CONTEXT_TURNS: usize = 6;

pub(super) fn conversation_match(
    addressing: &mut VoiceAddressing,
    entry: &ConversationEntry,
) -> Option<WakeMatch> {
    if !entry.from_the_microphone() {
        let holder_of = || addressing.boundaries.addressee();
        return greeted_in_conversation(addressing, &entry.text, &holder_of);
    }
    let named = live_sessions_named(addressing, &entry.text);
    let found = (!named.is_empty())
        .then(|| turned_to(addressing, entry, &named))
        .flatten();
    remember_turn(addressing, entry);
    found
}

fn remember_turn(addressing: &mut VoiceAddressing, entry: &ConversationEntry) {
    addressing.recent_turns.push_back(Turn {
        speaker: entry.speaker.clone(),
        text: entry.text.clone(),
    });
    if addressing.recent_turns.len() > CONTEXT_TURNS {
        addressing.recent_turns.pop_front();
    }
}

fn turned_to(
    addressing: &VoiceAddressing,
    entry: &ConversationEntry,
    named: &[Session],
) -> Option<WakeMatch> {
    let holder = addressing.boundaries.addressee();
    let candidates = by_called_name(&unheld(named, holder.as_ref()));
    let discord = addressing.boundaries.open_seat() == Some(Seat::Discord);
    match (candidates.is_empty(), discord) {
        (true, _) => None,
        (false, true) => discord_greeting(addressing, entry, holder.as_ref()),
        (false, false) => turned_utterance(addressing, entry, holder, &candidates),
    }
}

fn discord_greeting(
    addressing: &VoiceAddressing,
    entry: &ConversationEntry,
    holder: Option<&Addressee>,
) -> Option<WakeMatch> {
    let holder_of = || holder.cloned();
    greeted_in_conversation(addressing, &entry.text, &holder_of)
}

fn turned_utterance(
    addressing: &VoiceAddressing,
    entry: &ConversationEntry,
    holder: Option<Addressee>,
    candidates: &[(String, Session)],
) -> Option<WakeMatch> {
    let utterance = conversation_utterance(addressing, entry, holder);
    decided_or_greeted(addressing, &utterance, candidates)
}

fn conversation_utterance(
    addressing: &VoiceAddressing,
    entry: &ConversationEntry,
    holder: Option<Addressee>,
) -> Utterance {
    Utterance {
        phase: Phase::Conversation,
        text: entry.text.clone(),
        sequence: entry.sequence,
        holder,
        context: addressing.recent_turns.iter().cloned().collect(),
    }
}

fn decided_or_greeted(
    addressing: &VoiceAddressing,
    utterance: &Utterance,
    candidates: &[(String, Session)],
) -> Option<WakeMatch> {
    match decided(addressing, utterance, candidates) {
        None => {
            let holder_of = || utterance.holder.clone();
            greeted_in_conversation(addressing, &utterance.text, &holder_of)
        }
        Some(verdict) => chosen(addressing, &verdict, candidates, &utterance.text),
    }
}

fn unheld(named: &[Session], holder: Option<&Addressee>) -> Vec<Session> {
    named
        .iter()
        .filter(|session| !holds_the_floor(holder, session))
        .cloned()
        .collect()
}
