mod sinks;
mod targets;

use std::path::Path;
use std::sync::Arc;

use futures_util::future::BoxFuture;

pub use sinks::{PlayerPrograms, SinkPlayers};
pub use targets::{PlaybackTarget, playback_targets};

pub type StopAction = Arc<dyn Fn() + Send + Sync>;

pub struct RunningPlayback {
    pub stop: StopAction,
    pub finished: BoxFuture<'static, ()>,
}

pub trait Players: Send + Sync + 'static {
    fn start(&self, target: PlaybackTarget, audio_path: &Path) -> RunningPlayback;
}
