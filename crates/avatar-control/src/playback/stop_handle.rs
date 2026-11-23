use std::sync::{Arc, Mutex};

use tokio::sync::oneshot;

type StopSlot = Arc<Mutex<Option<oneshot::Sender<()>>>>;

#[derive(Debug, Clone, Default)]
pub struct StopHandle {
    slot: StopSlot,
}

impl StopHandle {
    pub(super) fn armed(signal: oneshot::Sender<()>) -> Self {
        Self {
            slot: Arc::new(Mutex::new(Some(signal))),
        }
    }

    pub fn stop(&self) {
        let _ = self
            .slot
            .lock()
            .map(|mut pending| pending.take().map(|signal| signal.send(())));
    }
}
