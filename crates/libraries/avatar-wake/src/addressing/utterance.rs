use avatar_floor::addressee::Addressee;
use avatar_routing::addressing::request::Turn;
use avatar_routing::names::session::Session;
use avatar_routing::names::spoken_names::called_name;

const MICROPHONE_SOURCE: &str = "human";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Wake,
    Conversation,
}

impl Phase {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Wake => "wake",
            Self::Conversation => "conversation",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Utterance {
    pub phase: Phase,
    pub text: String,
    pub sequence: Option<i64>,
    pub holder: Option<Addressee>,
    pub context: Vec<Turn>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConversationEntry {
    pub source: Option<String>,
    pub speaker: Option<String>,
    pub text: String,
    pub sequence: Option<i64>,
}

impl ConversationEntry {
    #[must_use]
    pub fn from_the_microphone(&self) -> bool {
        self.source.as_deref() == Some(MICROPHONE_SOURCE)
    }
}

#[must_use]
pub fn holds_the_floor(holder: Option<&Addressee>, session: &Session) -> bool {
    holder.is_some_and(|held| held.pane == session.pane && held.host == session.host)
}

#[must_use]
pub fn by_called_name(sessions: &[Session]) -> Vec<(String, Session)> {
    sessions.iter().fold(Vec::new(), with_called_name)
}

fn with_called_name(
    mut named: Vec<(String, Session)>,
    session: &Session,
) -> Vec<(String, Session)> {
    let Some(name) = called_name(&session.servant) else {
        return named;
    };
    match named.iter_mut().find(|(known, _)| *known == name) {
        Some(entry) => entry.1 = session.clone(),
        None => named.push((name, session.clone())),
    }
    named
}

impl Utterance {
    #[must_use]
    pub fn wake(text: &str) -> Self {
        Self {
            phase: Phase::Wake,
            text: text.to_owned(),
            sequence: None,
            holder: None,
            context: Vec::new(),
        }
    }
}
