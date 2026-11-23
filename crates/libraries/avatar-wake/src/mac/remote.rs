use avatar_floor::addressee::Addressee;
use avatar_floor::quiet::{Entry, ListenerPurpose};
use serde_json::Value;

use super::edges::MacEdges;
use crate::listener::{Boundary, BoundaryFailure};

pub(super) const FLOOR_TIMEOUT_SECONDS: u64 = 60;
pub(super) const CONVERSATION_READ_TIMEOUT_SECONDS: u64 = 15;

fn words(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_owned()).collect()
}

fn invalid(error: &serde_json::Error) -> BoundaryFailure {
    BoundaryFailure::Invalid(error.to_string())
}

pub(super) fn parsed_entries(output: &str) -> Boundary<Vec<Entry>> {
    output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).map_err(|error| invalid(&error)))
        .collect()
}

pub(super) fn read_entries(edges: &dyn MacEdges) -> Boundary<Vec<Entry>> {
    let command = words(&["avatar-listen", "read", "--since", "0"]);
    let output = edges.at_the_avatar_host(&command, CONVERSATION_READ_TIMEOUT_SECONDS)?;
    parsed_entries(&output)
}

pub(super) fn remote_purpose(edges: &dyn MacEdges) -> Boundary<Option<ListenerPurpose>> {
    let command = words(&["avatar-listen", "status"]);
    let output = edges.at_the_avatar_host(&command, CONVERSATION_READ_TIMEOUT_SECONDS)?;
    let status: Value = serde_json::from_str(&output).map_err(|error| invalid(&error))?;
    let listening = status
        .get("listening")
        .is_some_and(|value| value.as_bool() == Some(true));
    let purpose = status.get("purpose").cloned().unwrap_or(Value::Null);
    let parsed = serde_json::from_value(purpose).ok();
    Ok(parsed.filter(|_| listening))
}

#[must_use]
pub fn seen_from_this_mac(
    addressee: Addressee,
    room: Option<&str>,
    avatar_host: &str,
) -> Addressee {
    let host = match addressee.host.as_deref() {
        Some(host) if Some(host) == room => None,
        None => Some(avatar_host.to_owned()),
        Some(host) => Some(host.to_owned()),
    };
    Addressee { host, ..addressee }
}

pub(super) fn remote_addressee(edges: &dyn MacEdges) -> Boundary<Option<Addressee>> {
    let command = words(&["avatar-listen", "addressee"]);
    let output = edges.at_the_avatar_host(&command, CONVERSATION_READ_TIMEOUT_SECONDS)?;
    let holder: Option<Addressee> =
        serde_json::from_str(&output).map_err(|error| invalid(&error))?;
    let room = edges.room_name();
    Ok(holder.map(|held| seen_from_this_mac(held, room.as_deref(), &edges.avatar_host())))
}

fn claim_command(host: &str, addressee: &Addressee) -> Vec<String> {
    let agent = format!("{host}:{}", addressee.pane);
    let through = addressee.heard_through.to_string();
    words(&[
        "avatar-claim-floor",
        "--agent",
        &agent,
        "--heard-through",
        &through,
    ])
}

fn missing_room() -> BoundaryFailure {
    BoundaryFailure::Invalid("'AVATAR_AGENT_HOST'".to_owned())
}

pub(super) fn hand_to_the_avatar_host(edges: &dyn MacEdges, addressee: &Addressee) -> Boundary<()> {
    let named = addressee.host.clone().filter(|name| !name.is_empty());
    let host = named
        .or_else(|| edges.room_name())
        .ok_or_else(missing_room)?;
    let command = claim_command(&host, addressee);
    edges
        .at_the_avatar_host(&command, FLOOR_TIMEOUT_SECONDS)
        .map(|_| ())
}
