use std::path::Path;

use avatar_adapters::clock::wall_seconds;
use avatar_adapters::process::run_captured;
use avatar_floor::claim::{Conversation, Listening, RoomQuery, Site, parse_room_answer};
use std::io;
use std::time::Duration;

use super::capture::LocalCapture;
use super::commands::on_host;

const ROOM_QUERY_SECONDS: u64 = 5;

pub(crate) struct LocalListening<'capture>(pub(crate) &'capture dyn LocalCapture);

impl Listening for LocalListening<'_> {
    fn listening(&self) -> bool {
        self.0.listening()
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
    capture: &dyn LocalCapture,
    directory: &Path,
    host: &str,
    use_site: impl FnOnce(&Site<'_>) -> io::Result<Found>,
) -> io::Result<Found> {
    let listening = LocalListening(capture);
    use_site(&Site::new(directory, host, &listening, &Rooms))
}

pub(crate) fn claim_here(
    capture: &dyn LocalCapture,
    directory: &Path,
    claim: (&str, &str, &str),
) -> io::Result<bool> {
    let (host, pane, room) = claim;
    with_site(capture, directory, host, |site| {
        avatar_floor::claim::claim(site, pane, room, wall_seconds())
    })
}
