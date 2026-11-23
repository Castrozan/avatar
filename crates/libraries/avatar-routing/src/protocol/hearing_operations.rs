use serde::Deserialize;
use serde_json::{Value, json};

use crate::hearing::name_hearing::heard_name;
use crate::hearing::wake_phrase::{
    WakeMatch, addressed_session, named_in_turn, sessions_heard_anywhere, vocative_session,
};
use crate::names::session::Session;
use crate::names::vocabulary::Vocabulary;

#[derive(Deserialize)]
pub struct HeardNameRequest {
    pub windows: Vec<Vec<String>>,
    pub names: Vec<String>,
}

impl HeardNameRequest {
    #[must_use]
    pub fn answer(&self, vocabulary: Option<&Vocabulary>) -> Value {
        heard_name(&self.windows, &self.names, vocabulary).map_or(
            Value::Null,
            |(name, size)| json!({"name": name, "size": size}),
        )
    }
}

#[derive(Deserialize)]
pub struct SessionHearingRequest {
    pub text: String,
    pub sessions: Vec<Session>,
}

impl SessionHearingRequest {
    #[must_use]
    pub fn sessions_heard_anywhere(&self, vocabulary: Option<&Vocabulary>) -> Value {
        json!(sessions_heard_anywhere(
            &self.text,
            &self.sessions,
            vocabulary
        ))
    }

    #[must_use]
    pub fn vocative_session(&self, vocabulary: Option<&Vocabulary>) -> Value {
        json!(vocative_session(&self.text, &self.sessions, vocabulary))
    }

    #[must_use]
    pub fn addressed_session(&self, vocabulary: Option<&Vocabulary>) -> Value {
        wake_value(addressed_session(&self.text, &self.sessions, vocabulary))
    }

    #[must_use]
    pub fn named_in_turn(&self, vocabulary: Option<&Vocabulary>) -> Value {
        wake_value(named_in_turn(&self.text, &self.sessions, vocabulary))
    }
}

fn wake_value(found: Option<WakeMatch>) -> Value {
    found.map_or(
        Value::Null,
        |wake| json!({"session": wake.session, "message": wake.message}),
    )
}
