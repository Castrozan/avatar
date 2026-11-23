use std::path::Path;

use avatar_adapters::clock::wall_seconds;
use avatar_adapters::process::run_captured;
use avatar_floor::claim::{Conversation, Listening, RoomQuery, Site, parse_room_answer};
use avatar_listening::listener::runtime::listening;
use avatar_listening::listener::units::SystemdUnits;
use std::io;
use std::time::Duration;

use super::commands::on_host;

const ROOM_QUERY_SECONDS: u64 = 5;

pub(crate) struct LocalCapture;

impl Listening for LocalCapture {
    fn listening(&self) -> bool {
        listening(&mut SystemdUnits::new("systemctl"))
    }
}

pub(crate) struct Rooms;

impl RoomQuery for Rooms {
    fn conversation_in_room(&self, room: &str) -> Result<Option<Conversation>, String> {
        let command = on_host(Some(room), &["avatar-wake", "--conversation"]);
        let timeout = Duration::from_secs(ROOM_QUERY_SECONDS);
        let captured = run_captured(&command, &[], timeout).map_err(|error| error.to_string())?;
        let unverified = || format!("Cannot verify the conversation on {room}");
        let answer = captured
            .succeeded()
            .then_some(captured.stdout)
            .ok_or_else(unverified)?;
        parse_room_answer(room, &String::from_utf8_lossy(&answer))
    }
}

pub(crate) fn with_site<Found>(
    directory: &Path,
    host: &str,
    use_site: impl FnOnce(&Site<'_>) -> io::Result<Found>,
) -> io::Result<Found> {
    use_site(&Site::new(directory, host, &LocalCapture, &Rooms))
}

pub(crate) fn claim_here(directory: &Path, host: &str, pane: &str, room: &str) -> io::Result<bool> {
    with_site(directory, host, |site| {
        avatar_floor::claim::claim(site, pane, room, wall_seconds())
    })
}
