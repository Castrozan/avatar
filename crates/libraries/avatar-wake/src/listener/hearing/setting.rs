use avatar_floor::quiet::ListenerPurpose;
use avatar_routing::names::session::Session;
use serde_json::{Value, json};

use super::texts::{
    DESK_REPLY, LISTENING_LEAD, MEETING_SETTING, OPENING_LEAD, REMOTE_DESK_REPLY, REMOTE_MEETING,
    REMOTE_MEETING_LEAD, REMOTE_ROOM,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setting {
    Opening,
    Conversation,
    Meeting,
}

impl Setting {
    #[must_use]
    pub const fn of(purpose: Option<ListenerPurpose>) -> Option<Self> {
        match purpose {
            None => Some(Self::Opening),
            Some(ListenerPurpose::Conversation) => Some(Self::Conversation),
            Some(ListenerPurpose::Meeting) => Some(Self::Meeting),
            Some(ListenerPurpose::Live) => None,
        }
    }

    pub(in crate::listener) fn purpose_name(self) -> Value {
        match self {
            Self::Opening => Value::Null,
            Self::Conversation => json!("conversation"),
            Self::Meeting => json!("meeting"),
        }
    }

    fn local_text(self) -> String {
        match self {
            Self::Opening => [OPENING_LEAD, DESK_REPLY].concat(),
            Self::Conversation => [LISTENING_LEAD, DESK_REPLY].concat(),
            Self::Meeting => MEETING_SETTING.to_owned(),
        }
    }

    fn remote_text(self) -> String {
        match self {
            Self::Opening => [OPENING_LEAD, REMOTE_ROOM, REMOTE_DESK_REPLY].concat(),
            Self::Conversation => [LISTENING_LEAD, REMOTE_ROOM, REMOTE_DESK_REPLY].concat(),
            Self::Meeting => [REMOTE_MEETING_LEAD, REMOTE_ROOM, REMOTE_MEETING].concat(),
        }
    }
}

fn setting_for(setting: Setting, session: &Session, room: &str) -> String {
    if session.host.is_none() {
        return setting.local_text();
    }
    let reach = format!("AVATAR_ROOM={room}");
    let template = setting.remote_text();
    template.replace("{room}", room).replace("{reach}", &reach)
}

#[must_use]
pub fn spoken_prompt(message: &str, setting: Setting, session: &Session, room: &str) -> String {
    let words = setting_for(setting, session, room);
    format!("[wake] Voice from the room, speaker not verified: {message} {words}")
}
