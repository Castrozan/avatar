mod commands;
mod finished;
mod transcript;
mod utterance;
mod watch;

pub use commands::{ask, follow, hear, listen_after};
pub use transcript::{TRANSCRIPT_FILE, read_since, wait_since};
pub use utterance::next_utterance;
pub use watch::{Addressing, ListeningWatch};

pub const POLL_SECONDS: f64 = 0.1;
pub const VOICE_TAG: &str = "[voice]";
