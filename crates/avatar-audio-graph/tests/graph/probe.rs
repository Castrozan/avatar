use std::io;

use avatar_audio_graph::state::record::AudioState;
use avatar_audio_graph::state::store::StateStore;

use super::replay::Probe;

pub(crate) struct ProbeStore {
    probe: Probe,
}

impl ProbeStore {
    pub(crate) fn new(probe: Probe) -> Self {
        Self { probe }
    }
}

impl StateStore for ProbeStore {
    fn save(&mut self, state: &AudioState) -> io::Result<()> {
        *self.probe.borrow_mut() = Some(state.clone());
        Ok(())
    }

    fn remove(&mut self) -> io::Result<()> {
        *self.probe.borrow_mut() = None;
        Ok(())
    }
}
