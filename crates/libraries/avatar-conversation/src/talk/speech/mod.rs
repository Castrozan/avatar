mod say;
mod spoken;
mod turn;

pub use say::{reply, say, speak_on_seat, speech_argv};
pub use spoken::{CompletedSpeech, spoken_in_turn};

use crate::talk::ports::Files;

pub const SPEECH_TIMEOUT_SECONDS: f64 = 310.0;
const SPOKEN_FILE: &str = "spoken";

pub fn mark_spoken(files: &mut impl Files) {
    files.touch(SPOKEN_FILE);
}
