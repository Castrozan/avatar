mod choice;
mod wait;

pub use choice::{live_voices_by_character, voice_for};
pub use wait::{HOLDER_POLL_SECONDS, HOLDER_WAIT_SECONDS, voice_of_the_room_holder};
