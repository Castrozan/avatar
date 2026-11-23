use avatar_floor::quiet::Seat;
use avatar_routing::names::session::Session;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::reach::{CommandRunner, ssh_command};

pub const PEER_TIMEOUT_SECONDS: u64 = 10;
pub const DISCORD_SEAT: &str = "discord";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeerListing {
    pub sessions: Vec<Session>,
    pub conversation_seat: Option<String>,
}

impl PeerListing {
    #[must_use]
    pub fn is_on_discord(&self) -> bool {
        self.conversation_seat.as_deref() == Some(DISCORD_SEAT)
    }
}

#[derive(Deserialize)]
struct WireListing {
    sessions: Vec<WireSession>,
    conversation_seat: Value,
}

#[derive(Deserialize)]
struct WireSession {
    servant: String,
    pane: String,
}

#[must_use]
pub fn peer_listing_command(host: &str) -> Vec<String> {
    ssh_command(host, &["avatar-wake", "--sessions"])
}

#[must_use]
pub fn parse_listing(host: &str, reply: &str) -> Option<PeerListing> {
    let wire: WireListing = serde_json::from_str(reply).ok()?;
    Some(PeerListing {
        sessions: wire
            .sessions
            .into_iter()
            .map(|entry| peer_session(host, entry))
            .collect(),
        conversation_seat: wire.conversation_seat.as_str().map(str::to_owned),
    })
}

fn peer_session(host: &str, entry: WireSession) -> Session {
    Session {
        servant: entry.servant,
        pane: entry.pane,
        host: Some(host.to_owned()),
    }
}

#[must_use]
pub fn listing_of(sessions: &[Session], conversation_seat: Option<Seat>) -> String {
    let entries: Vec<Value> = sessions
        .iter()
        .map(|session| json!({"servant": session.servant, "pane": session.pane}))
        .collect();
    json!({"sessions": entries, "conversation_seat": conversation_seat}).to_string()
}

#[must_use]
pub fn sessions_of_peer(host: &str, runner: &dyn CommandRunner) -> Option<PeerListing> {
    let reply = runner.stdout_of(&peer_listing_command(host), PEER_TIMEOUT_SECONDS)?;
    parse_listing(host, &reply)
}

#[must_use]
pub fn sessions_everywhere(local_sessions: &[Session], peer_sessions: &[Session]) -> Vec<Session> {
    let remote = peer_sessions.iter().filter(|peer| {
        !local_sessions
            .iter()
            .any(|local| local.servant == peer.servant)
    });
    local_sessions
        .iter()
        .cloned()
        .chain(remote.cloned())
        .collect()
}
