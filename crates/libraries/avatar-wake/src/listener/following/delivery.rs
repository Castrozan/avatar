use avatar_floor::quiet::Entry;
use avatar_routing::hearing::wake_phrase::WakeMatch;

use super::MEETING_SOURCE;
use crate::addressing::ConversationEntry;
use crate::listener::boundaries::Boundaries;
use crate::listener::failure::Boundary;
use crate::listener::hearing::addressee_of;
use crate::listener::hearing::prompts::{Dispatch, deliver};
use crate::listener::hearing::setting::Setting;

pub(super) fn hand_and_deliver(
    dry_run: bool,
    boundaries: Boundaries,
    (matched, turn): (&WakeMatch, &Entry),
) -> Boundary<()> {
    if !dry_run {
        let addressee = addressee_of(&matched.session, turn.sequence);
        boundaries.seat.hand_to(&addressee)?;
    }
    deliver_to_the_purpose(dry_run, boundaries, (matched, turn))
}

fn deliver_to_the_purpose(
    dry_run: bool,
    boundaries: Boundaries,
    (matched, turn): (&WakeMatch, &Entry),
) -> Boundary<()> {
    let Some(setting) = setting_of(boundaries, turn)? else {
        return Ok(());
    };
    let dispatch = Dispatch {
        dry_run,
        matched,
        setting,
        sequence: Some(turn.sequence),
    };
    deliver(boundaries.effects, &dispatch).map(|_| ())
}

fn setting_of(boundaries: Boundaries, turn: &Entry) -> Boundary<Option<Setting>> {
    let purpose = boundaries.seat.listening_purpose()?;
    Ok(Setting::of(purpose).filter(|_| !from_the_meeting(turn)))
}

fn from_the_meeting(turn: &Entry) -> bool {
    turn.source.as_deref() == Some(MEETING_SOURCE)
}

pub(super) fn conversation_entry(turn: &Entry) -> ConversationEntry {
    ConversationEntry {
        source: turn.source.clone(),
        speaker: turn.speaker.clone(),
        text: turn.text.clone(),
        sequence: Some(turn.sequence),
    }
}
