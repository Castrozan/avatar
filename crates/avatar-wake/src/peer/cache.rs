use avatar_routing::names::session::Session;

use super::listing::{PeerListing, sessions_of_peer};
use crate::reach::CommandRunner;

pub const PEER_REFRESH_SECONDS: f64 = 20.0;

#[derive(Debug, Default)]
pub struct PeerCache {
    peer_hosts: Vec<String>,
    sessions: Vec<Session>,
    discord_hosts: Vec<String>,
    answering: Vec<String>,
    started_at: Option<f64>,
    refreshing: bool,
}

impl PeerCache {
    #[must_use]
    pub fn new(peer_hosts: Vec<String>) -> Self {
        Self {
            peer_hosts,
            ..Self::default()
        }
    }

    pub fn begin_refresh_if_due(&mut self, now: f64) -> bool {
        begin_if_due(self, now)
    }

    pub fn refresh(&mut self, runner: &dyn CommandRunner) {
        let listings: Vec<(String, Option<PeerListing>)> = unique(&self.peer_hosts)
            .into_iter()
            .map(|host| (host.clone(), sessions_of_peer(&host, runner)))
            .collect();
        self.apply(&listings);
        self.refreshing = false;
    }

    fn apply(&mut self, listings: &[(String, Option<PeerListing>)]) {
        self.sessions = listings
            .iter()
            .filter_map(|(_, listing)| listing.as_ref())
            .flat_map(|listing| listing.sessions.clone())
            .collect();
        self.discord_hosts = hosts_where(listings, PeerListing::is_on_discord);
        self.answering = answering_hosts_of(listings);
    }

    #[must_use]
    pub fn sessions(&self) -> &[Session] {
        &self.sessions
    }

    #[must_use]
    pub fn discord_call_host(&self) -> Option<&str> {
        self.discord_hosts.first().map(String::as_str)
    }

    #[must_use]
    pub fn answering_hosts(&self) -> &[String] {
        &self.answering
    }
}

fn begin_if_due(cache: &mut PeerCache, now: f64) -> bool {
    if cache.peer_hosts.is_empty() || cache.refreshing || started_within_interval(cache, now) {
        return false;
    }
    cache.refreshing = true;
    cache.started_at = Some(now);
    true
}

fn started_within_interval(cache: &PeerCache, now: f64) -> bool {
    cache
        .started_at
        .is_some_and(|started| now - started < PEER_REFRESH_SECONDS)
}

fn unique(hosts: &[String]) -> Vec<String> {
    hosts
        .iter()
        .enumerate()
        .filter(|(index, host)| !hosts[..*index].contains(host))
        .map(|(_, host)| host.clone())
        .collect()
}

fn hosts_where(
    listings: &[(String, Option<PeerListing>)],
    matches: impl Fn(&PeerListing) -> bool,
) -> Vec<String> {
    listings
        .iter()
        .filter(|(_, listing)| listing.as_ref().is_some_and(&matches))
        .map(|(host, _)| host.clone())
        .collect()
}

fn answering_hosts_of(listings: &[(String, Option<PeerListing>)]) -> Vec<String> {
    listings
        .iter()
        .filter(|(_, listing)| listing.is_some())
        .map(|(host, _)| host.clone())
        .collect()
}
