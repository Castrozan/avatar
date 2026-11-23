use super::conversation::{Conversation, Listening, RoomQuery};
use super::json::conversation_text;
use crate::addressee::{Addressee, current};
use std::io;
use std::path::Path;

pub struct Site<'a> {
    pub(super) directory: &'a Path,
    pub(super) host: &'a str,
    pub(super) capture: &'a dyn Listening,
    pub(super) rooms: &'a dyn RoomQuery,
}

impl<'a> Site<'a> {
    #[must_use]
    pub fn new(
        directory: &'a Path,
        host: &'a str,
        capture: &'a dyn Listening,
        rooms: &'a dyn RoomQuery,
    ) -> Self {
        Self {
            directory,
            host,
            capture,
            rooms,
        }
    }
}

pub fn conversation_report(site: &Site<'_>) -> io::Result<String> {
    let conversation = active_conversation(site)?;
    Ok(conversation_text(conversation.as_ref()))
}

pub(super) fn active_conversation(site: &Site<'_>) -> io::Result<Option<Conversation>> {
    let Some(addressee) = current(site.directory)? else {
        return Ok(None);
    };
    if !site.capture.listening() {
        return Ok(None);
    }
    Ok(Some(conversation_of(site.host, addressee)))
}

fn conversation_of(host: &str, addressee: Addressee) -> Conversation {
    Conversation {
        host: addressee
            .host
            .filter(|named| !named.is_empty())
            .unwrap_or_else(|| host.to_owned()),
        pane: addressee.pane,
    }
}

pub(super) fn conversation_in_room(
    site: &Site<'_>,
    room: &str,
) -> io::Result<Option<Conversation>> {
    if room == site.host {
        return active_conversation(site);
    }
    site.rooms
        .conversation_in_room(room)
        .map_err(io::Error::other)
}
