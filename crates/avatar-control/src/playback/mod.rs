mod command;
mod handle;
mod program;
mod pulse_sink;

use std::path::Path;

pub use handle::{Playback, PlaybackReport};
pub use program::ProgramAudioPlayer;
pub use pulse_sink::PulseSinkPlayer;

pub trait AudioPlayer {
    fn play(&self, audio_path: &Path) -> Playback;
}
