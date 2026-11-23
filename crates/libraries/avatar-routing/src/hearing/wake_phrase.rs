use crate::hearing::wake_listener::WakeListener;
use crate::hearing::wake_windows::is_greeting;
use crate::names::session::Session;
use crate::names::vocabulary::Vocabulary;
use crate::text::folding::{folded, word_matches};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeMatch {
    pub session: Session,
    pub message: String,
}

#[must_use]
pub fn opens_with_greeting(text: &str) -> bool {
    word_matches(text)
        .first()
        .is_some_and(|first| is_greeting(&folded(&first.text)))
}

#[must_use]
pub fn sessions_heard_anywhere(
    text: &str,
    sessions: &[Session],
    vocabulary: Option<&Vocabulary>,
) -> Vec<Session> {
    WakeListener::new(sessions, vocabulary).sessions_heard_anywhere(text)
}

#[must_use]
pub fn vocative_session(
    text: &str,
    sessions: &[Session],
    vocabulary: Option<&Vocabulary>,
) -> Option<Session> {
    WakeListener::new(sessions, vocabulary)
        .vocative(text)
        .map(|wake| wake.session)
}

#[must_use]
pub fn addressed_session(
    text: &str,
    sessions: &[Session],
    vocabulary: Option<&Vocabulary>,
) -> Option<WakeMatch> {
    WakeListener::new(sessions, vocabulary).addressed(text)
}

#[must_use]
pub fn named_in_turn(
    text: &str,
    sessions: &[Session],
    vocabulary: Option<&Vocabulary>,
) -> Option<WakeMatch> {
    WakeListener::new(sessions, vocabulary).named_in_turn(text)
}
