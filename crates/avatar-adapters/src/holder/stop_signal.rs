use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::runtime::Handle;
use tokio::signal::unix::{Signal, SignalKind, signal};

#[derive(Clone, Debug, Default)]
pub struct StopSignal {
    raised: Arc<AtomicBool>,
}

impl StopSignal {
    #[must_use]
    pub fn is_raised(&self) -> bool {
        self.raised.load(Ordering::SeqCst)
    }

    pub fn raise(&self) {
        self.raised.store(true, Ordering::SeqCst);
    }

    pub fn on_sigterm(runtime: &Handle) -> io::Result<Self> {
        let _entered = runtime.enter();
        let terminate = signal(SignalKind::terminate())?;
        let stop = Self::default();
        let raiser = stop.clone();
        runtime.spawn(raise_on_terminate(terminate, raiser));
        Ok(stop)
    }
}

async fn raise_on_terminate(mut terminate: Signal, stop: StopSignal) {
    terminate.recv().await;
    stop.raise();
}
