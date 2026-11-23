mod begin;
mod directory;
mod error;

use crate::conversation_state::ListenerPurpose;
use crate::listener::runtime::{LISTENER_SERVICE, LIVE_SERVICE};

pub use begin::{SessionRequest, begin_listening, live_purpose, stop_listening};
pub use directory::{
    CONFIGURATION_FILE, EMBEDDINGS_FILE, HEARD_FILE, SESSION_RECORDS, SPEECH_IN_PROGRESS_MARKER,
    SPOKEN_FILE, SessionDirectory, TRANSCRIPT_FILE, quiet_seconds_since,
};
pub use error::SessionError;

#[must_use]
pub const fn service_for(purpose: ListenerPurpose) -> &'static str {
    match purpose {
        ListenerPurpose::Conversation | ListenerPurpose::Meeting => LISTENER_SERVICE,
        ListenerPurpose::Live => LIVE_SERVICE,
    }
}

#[must_use]
pub const fn purpose_name(purpose: ListenerPurpose) -> &'static str {
    match purpose {
        ListenerPurpose::Conversation => "conversation",
        ListenerPurpose::Meeting => "meeting",
        ListenerPurpose::Live => "live",
    }
}
