use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;

use avatar_adapters::clock::monotonic_seconds;
use avatar_routing::names::session::Session;
use avatar_wake::peer::cache::{PEER_REFRESH_SECONDS, PeerCache};

use super::commands::Shell;

const HOSTS_VARIABLE: &str = "AVATAR_PEER_HOSTS";

pub(crate) struct Peers {
    hosts: Vec<String>,
    cache: Arc<Mutex<PeerCache>>,
    started_at: Mutex<Option<f64>>,
    working: Arc<AtomicBool>,
}

fn locked<Held>(mutex: &Mutex<Held>) -> MutexGuard<'_, Held> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn due(peers: &Peers) -> bool {
    let started = *locked(&peers.started_at);
    let recent = started.is_some_and(|at| monotonic_seconds() - at < PEER_REFRESH_SECONDS);
    !peers.hosts.is_empty() && !recent && !peers.working.swap(true, Ordering::SeqCst)
}

fn refresh_when_due(peers: &Peers) {
    if due(peers) {
        *locked(&peers.started_at) = Some(monotonic_seconds());
        spawn_refresh(peers);
    }
}

fn spawn_refresh(peers: &Peers) {
    let (cache, working) = (peers.cache.clone(), peers.working.clone());
    let hosts = peers.hosts.clone();
    thread::spawn(move || {
        let mut fresh = PeerCache::new(hosts);
        fresh.refresh(&Shell);
        *locked(&cache) = fresh;
        working.store(false, Ordering::SeqCst);
    });
}

impl Peers {
    pub(crate) fn from_environment() -> Self {
        let configured = std::env::var(HOSTS_VARIABLE).unwrap_or_default();
        let hosts: Vec<String> = configured.split_whitespace().map(str::to_owned).collect();
        Self {
            cache: Arc::new(Mutex::new(PeerCache::new(hosts.clone()))),
            hosts,
            started_at: Mutex::new(None),
            working: Arc::new(AtomicBool::new(false)),
        }
    }

    pub(crate) fn sessions(&self) -> Vec<Session> {
        refresh_when_due(self);
        locked(&self.cache).sessions().to_vec()
    }

    pub(crate) fn discord_call_host(&self) -> Option<String> {
        refresh_when_due(self);
        locked(&self.cache).discord_call_host().map(str::to_owned)
    }

    pub(crate) fn answering_hosts(&self) -> Vec<String> {
        refresh_when_due(self);
        locked(&self.cache).answering_hosts().to_vec()
    }
}
