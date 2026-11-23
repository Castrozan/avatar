use std::path::PathBuf;
use std::time::Duration;

use super::command::run_checked;
use crate::client::Speakers;
use crate::failure::Failure;

const PLAYBACK_TIMEOUT: Duration = Duration::from_secs(310);
const AFPLAY: &str = "/usr/bin/afplay";

#[derive(Debug, Clone)]
pub struct Afplay {
    player: PathBuf,
}

impl Afplay {
    #[must_use]
    pub fn new(player: impl Into<PathBuf>) -> Self {
        Self {
            player: player.into(),
        }
    }

    #[must_use]
    pub fn system() -> Self {
        Self::new(AFPLAY)
    }
}

impl Speakers for Afplay {
    fn play(&mut self, recording: &str) -> Result<(), Failure> {
        run_checked(&self.player, &[recording.to_owned()], PLAYBACK_TIMEOUT)
    }
}
