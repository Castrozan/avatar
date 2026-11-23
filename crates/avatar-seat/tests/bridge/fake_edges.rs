use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use avatar_seat::bridge::BridgeLog;
use avatar_seat::host::levels::{LevelSource, Sink, SinkReadFailure};
use avatar_seat::host::state::{HostProbe, HostSnapshot, ProbeFailure};

#[derive(Clone, Default)]
pub(crate) struct FakeLevels {
    pub(crate) sinks: Arc<Mutex<Vec<Sink>>>,
    pub(crate) reads: Arc<AtomicUsize>,
}

impl LevelSource for FakeLevels {
    async fn read_sinks(&self) -> Result<Vec<Sink>, SinkReadFailure> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        self.sinks
            .lock()
            .map(|sinks| sinks.clone())
            .map_err(|_| SinkReadFailure)
    }
}

pub(crate) struct FakeHost;

impl HostProbe for FakeHost {
    fn read(&self) -> Result<HostSnapshot, ProbeFailure> {
        Ok(HostSnapshot {
            listening: false,
            meeting: true,
        })
    }
}

pub(crate) struct Silent;

impl BridgeLog for Silent {
    fn log(&self, _message: &str) {}
}
