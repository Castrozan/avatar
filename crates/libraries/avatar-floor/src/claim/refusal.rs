use super::conversation::matches_session;
use super::record::{held_by_another_room, recorded_claim};
use super::site::{Site, active_conversation, conversation_in_room};
use std::io;

pub(super) fn refused(
    site: &Site<'_>,
    pane: &str,
    room: &str,
    now: f64,
    recorded: &str,
) -> io::Result<bool> {
    Ok(session_talks_on_this_machine(site, pane, room)?
        || held_by_another_room(recorded, room, now)
        || session_talks_in_the_previous_room(site, pane, room, recorded)?)
}

fn session_talks_on_this_machine(site: &Site<'_>, pane: &str, room: &str) -> io::Result<bool> {
    if room == site.host {
        return Ok(false);
    }
    let conversation = active_conversation(site)?;
    Ok(matches_session(conversation.as_ref(), site.host, pane))
}

fn session_talks_in_the_previous_room(
    site: &Site<'_>,
    pane: &str,
    room: &str,
    recorded: &str,
) -> io::Result<bool> {
    let claim = recorded_claim(recorded);
    let Some(previous_room) = claim.other_holder(room) else {
        return Ok(false);
    };
    let conversation = conversation_in_room(site, previous_room)?;
    Ok(matches_session(conversation.as_ref(), site.host, pane))
}
