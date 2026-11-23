use std::path::PathBuf;

use avatar_adapters::holder::StopSignal;
use avatar_adapters::holder::StoppableClock;
use avatar_conversation::talk::state::configuration::read_configuration;
use avatar_holder::default_holder::{DefaultHolder, configured_holder};
use avatar_holder::floor_holder::{FloorHolder, HoldingInterruption};
use avatar_holder::holder_cli::ConversationSession;

use super::floor::HolderRoom;
use super::voice::HolderVoice;
use crate::talk::TalkBoundaries;

pub(super) struct RuntimeSession {
    directory: PathBuf,
    stop: StopSignal,
}

impl RuntimeSession {
    pub(super) const fn new(directory: PathBuf, stop: StopSignal) -> Self {
        Self { directory, stop }
    }

    fn boundaries(&self) -> TalkBoundaries {
        TalkBoundaries::on_this_machine(self.directory.clone())
    }
}

impl ConversationSession for RuntimeSession {
    fn configured_holder(&mut self) -> Result<Option<DefaultHolder>, String> {
        let configuration = read_configuration(&mut self.boundaries());
        let record = configuration.map_err(|failure| failure.to_string())?;
        configured_holder(&record).map_err(|failure| failure.to_string())
    }

    fn hold_floor(&mut self, holder: &DefaultHolder) -> HoldingInterruption {
        let clock = StoppableClock::new(self.stop.clone());
        let room = HolderRoom::new(self.boundaries(), clock, holder.pane());
        let voice = HolderVoice::new(holder.profile(), &self.directory);
        FloorHolder::new(holder.clone(), room, voice).hold()
    }
}
