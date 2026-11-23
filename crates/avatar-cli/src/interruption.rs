use std::io;

use tokio::signal::unix::{Signal, SignalKind, signal};

pub(crate) struct Interruption {
    interrupt: Signal,
    terminate: Signal,
}

impl Interruption {
    pub(crate) fn listen() -> io::Result<Self> {
        Ok(Self {
            interrupt: signal(SignalKind::interrupt())?,
            terminate: signal(SignalKind::terminate())?,
        })
    }

    pub(crate) async fn arrived(&mut self) {
        tokio::select! {
            _ = self.interrupt.recv() => (),
            _ = self.terminate.recv() => (),
        }
    }
}
