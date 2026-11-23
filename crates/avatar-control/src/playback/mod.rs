mod command;
mod handle;
mod program;
mod pulse_sink;
mod stop_handle;

use std::path::Path;

pub use handle::{Playback, PlaybackReport};
pub use program::ProgramAudioPlayer;
pub use pulse_sink::PulseSinkPlayer;
pub use stop_handle::StopHandle;

pub trait AudioPlayer {
    fn play(&self, audio_path: &Path) -> Playback;
}
