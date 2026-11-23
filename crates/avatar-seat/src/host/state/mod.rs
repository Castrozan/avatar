use std::sync::Arc;
use std::time::Duration;

use tokio::sync::watch;

pub const MEETING_TARGET: &str = "avatar.target";
pub const REFRESH_SECONDS: u64 = 2;
pub const READ_TIMEOUT_SECONDS: u64 = 45;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HostSnapshot {
    pub listening: bool,
    pub meeting: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeFailure;

pub trait HostProbe: Send + Sync + 'static {
    fn read(&self) -> Result<HostSnapshot, ProbeFailure>;
}

pub struct HostState<Probe: HostProbe> {
    probe: Arc<Probe>,
    known: watch::Sender<HostSnapshot>,
}

async fn read_within_limit<Probe: HostProbe>(probe: Arc<Probe>) -> Option<HostSnapshot> {
    let read = tokio::task::spawn_blocking(move || probe.read());
    let limit = Duration::from_secs(READ_TIMEOUT_SECONDS);
    tokio::time::timeout(limit, read).await.ok()?.ok()?.ok()
}

async fn follow_forever<Probe: HostProbe>(state: &HostState<Probe>) {
    loop {
        state.refresh().await;
        tokio::time::sleep(Duration::from_secs(REFRESH_SECONDS)).await;
    }
}

impl<Probe: HostProbe> HostState<Probe> {
    pub fn new(probe: Probe) -> Self {
        let (known, _) = watch::channel(HostSnapshot::default());
        Self {
            probe: Arc::new(probe),
            known,
        }
    }

    #[must_use]
    pub fn snapshot(&self) -> HostSnapshot {
        *self.known.borrow()
    }

    pub async fn refresh(&self) {
        let read = read_within_limit(Arc::clone(&self.probe)).await;
        read.map(|snapshot| self.known.send_replace(snapshot));
    }

    pub async fn follow(&self) {
        follow_forever(self).await;
    }
}
