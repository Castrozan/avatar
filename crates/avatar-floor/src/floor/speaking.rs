use super::session::Session;
use crate::addressee::{Addressee, current, exclusive, holds_without_a_pane, write};
use std::io;
use std::path::Path;

use super::boundaries::FloorBoundaries;

#[must_use]
pub fn first_claim(
    pane: &str,
    sessions: &[Session],
    holder: Option<&Addressee>,
) -> Option<Addressee> {
    let claimer = sessions.iter().find(|session| session.pane == pane)?;
    Some(Addressee {
        pane: pane.to_owned(),
        servant: claimer.servant.clone(),
        heard_through: holder.map_or(0, |held| held.heard_through),
        host: None,
    })
}

#[must_use]
pub fn holder_has_a_live_session(holder: Option<&Addressee>, sessions: &[Session]) -> bool {
    holder.is_some_and(|held| {
        holds_without_a_pane(held) || sessions.iter().any(|session| session.pane == held.pane)
    })
}

pub fn claim_among_live_sessions(
    directory: &Path,
    pane: &str,
    holder: Option<&Addressee>,
    sessions: &[Session],
) -> io::Result<Option<Addressee>> {
    if holder_has_a_live_session(holder, sessions) {
        return Ok(holder.cloned());
    }
    let claimer = first_claim(pane, sessions, holder);
    if let Some(claimer) = &claimer {
        write(directory, claimer)?;
    }
    Ok(claimer)
}

pub fn claim(
    directory: &Path,
    pane: &str,
    boundaries: &dyn FloorBoundaries,
) -> io::Result<Option<Addressee>> {
    let _lock = exclusive(directory)?;
    let holder = current(directory)?;
    if holder.as_ref().is_some_and(|held| held.pane == pane) {
        return Ok(holder);
    }
    if !boundaries.listening() {
        return Ok(None);
    }
    claim_among_live_sessions(directory, pane, holder.as_ref(), &boundaries.sessions())
}

#[must_use]
pub fn may_speak(holder: Option<&Addressee>, pane: &str) -> bool {
    holder.is_none_or(|held| held.pane == pane)
}
