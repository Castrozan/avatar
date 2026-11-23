use crate::hearing::name_hearing::heard_name;
use crate::hearing::spoken_name_index::sessions_by_spoken_name;
use crate::hearing::wake_phrase::WakeMatch;
use crate::hearing::wake_windows::{
    folded_texts, greeting_lead, leading_greetings, leading_windows, push_distinct,
    trailing_windows,
};
use crate::names::session::Session;
use crate::names::vocabulary::Vocabulary;
use crate::text::folding::word_matches;

const MESSAGE_EDGE: &[char] = &[' ', ',', '.', ';', ':', '!', '?', '-'];

pub struct WakeListener<'a> {
    candidates: Vec<(String, Session)>,
    vocabulary: Option<&'a Vocabulary>,
}

impl<'a> WakeListener<'a> {
    #[must_use]
    pub fn new(sessions: &[Session], vocabulary: Option<&'a Vocabulary>) -> Self {
        Self {
            candidates: sessions_by_spoken_name(sessions),
            vocabulary,
        }
    }

    #[must_use]
    pub fn sessions_heard_anywhere(&self, text: &str) -> Vec<Session> {
        let texts = folded_texts(&word_matches(text));
        (0..texts.len())
            .filter_map(|start| self.hear(&leading_windows(&texts, start)))
            .map(|(session, _)| session)
            .fold(Vec::new(), push_distinct)
    }

    #[must_use]
    pub fn vocative(&self, text: &str) -> Option<WakeMatch> {
        let texts = folded_texts(&word_matches(text));
        let windows = vocative_windows(&texts)?;
        let (session, _) = self.hear(&windows)?;
        Some(wake_match(session, text))
    }

    #[must_use]
    pub fn addressed(&self, text: &str) -> Option<WakeMatch> {
        let words = word_matches(text);
        let texts = folded_texts(&words);
        let greetings = greeting_lead(&texts)?;
        let (session, size) = self.hear(&leading_windows(&texts, greetings))?;
        let last_heard = words.get(greetings + size - 1)?;
        Some(wake_match(session, &text[last_heard.end..]))
    }

    #[must_use]
    pub fn named_in_turn(&self, text: &str) -> Option<WakeMatch> {
        self.addressed(text).or_else(|| self.vocative(text))
    }

    fn hear(&self, windows: &[Vec<String>]) -> Option<(Session, usize)> {
        let names: Vec<String> = self
            .candidates
            .iter()
            .map(|(name, _)| name.clone())
            .collect();
        let (heard, size) = heard_name(windows, &names, self.vocabulary)?;
        Some((self.session_named(&heard)?, size))
    }

    fn session_named(&self, name: &str) -> Option<Session> {
        self.candidates
            .iter()
            .find(|(candidate, _)| candidate.as_str() == name)
            .map(|(_, session)| session.clone())
    }
}

fn vocative_windows(texts: &[String]) -> Option<Vec<Vec<String>>> {
    let greetings = leading_greetings(texts);
    if greetings == texts.len() {
        return None;
    }
    let mut windows = leading_windows(texts, greetings);
    windows.extend(trailing_windows(texts, greetings));
    Some(windows)
}

fn wake_match(session: Session, raw_message: &str) -> WakeMatch {
    WakeMatch {
        session,
        message: raw_message.trim_matches(MESSAGE_EDGE).to_owned(),
    }
}
