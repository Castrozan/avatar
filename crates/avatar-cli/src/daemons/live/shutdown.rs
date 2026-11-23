use std::io;
use std::sync::{Arc, Mutex, PoisonError};

use tokio::signal::unix::{SignalKind, signal};
use tokio::sync::watch;
use tokio::task::spawn_local;

pub(crate) struct Shutdown {
    requested: watch::Sender<bool>,
    failure: Mutex<Option<String>>,
}

impl Shutdown {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            requested: watch::channel(false).0,
            failure: Mutex::new(None),
        })
    }

    pub(crate) fn request(&self) {
        self.requested.send_replace(true);
    }

    pub(crate) fn fail(&self, message: String) {
        let mut failure = self.failure.lock().unwrap_or_else(PoisonError::into_inner);
        failure.get_or_insert(message);
        drop(failure);
        self.request();
    }

    pub(crate) async fn requested(&self) {
        let mut receiver = self.requested.subscribe();
        let _ = receiver.wait_for(|stopped| *stopped).await;
    }

    pub(crate) fn failure(&self) -> Option<String> {
        let failure = self.failure.lock().unwrap_or_else(PoisonError::into_inner);
        failure.clone()
    }
}

fn stop_on(kind: SignalKind, shutdown: &Arc<Shutdown>) -> io::Result<()> {
    let mut received = signal(kind)?;
    let shutdown = Arc::clone(shutdown);
    spawn_local(async move {
        received.recv().await;
        shutdown.request();
    });
    Ok(())
}

pub(crate) fn stop_on_signals(shutdown: &Arc<Shutdown>) -> io::Result<()> {
    stop_on(SignalKind::interrupt(), shutdown)?;
    stop_on(SignalKind::terminate(), shutdown)?;
    stop_on(SignalKind::hangup(), shutdown)
}
