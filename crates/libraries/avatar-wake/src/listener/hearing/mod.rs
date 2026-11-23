use avatar_routing::names::session::Session;

use super::boundaries::{Boundaries, WakeEffects};
use super::bridge::with_addressing;
use super::failure::Boundary;
use super::machine::WakeListener;
use super::records::report_wake;
use prompts::{Dispatch, deliver};
use setting::Setting;
use transcribing::transcribed;

pub(super) mod prompts;
pub(super) mod setting;
mod texts;
mod transcribing;
use crate::arbitration::room_holding::{HeardSegment, holds_the_room};
use avatar_floor::addressee::Addressee;
use avatar_routing::hearing::wake_phrase::WakeMatch;

pub(super) fn addressee_of(session: &Session, heard_through: i64) -> Addressee {
    Addressee {
        pane: session.pane.clone(),
        servant: session.servant.clone(),
        heard_through,
        host: session.host.clone(),
    }
}

fn byte_count(segment: &[u8]) -> u32 {
    u32::try_from(segment.len()).unwrap_or(u32::MAX)
}

pub(super) fn hear(listener: &mut WakeListener, boundaries: Boundaries, segment: &[u8]) {
    if let Err(failure) = answer(listener, boundaries, segment) {
        let line = format!("Dropped an utterance: {}", failure.message());
        boundaries.effects.report(&line);
    }
}

fn answer(listener: &mut WakeListener, boundaries: Boundaries, segment: &[u8]) -> Boundary<()> {
    let heard_at = boundaries.effects.now();
    let heard = transcribed(boundaries.effects, segment)?;
    let audio = HeardSegment {
        audio_byte_count: byte_count(segment),
        heard_at,
    };
    if !reported_nonempty(boundaries.effects, &heard, &audio) {
        return Ok(());
    }
    wake(listener, boundaries, (&heard, &audio))
}

fn reported_nonempty(effects: &dyn WakeEffects, heard: &str, audio: &HeardSegment) -> bool {
    report_wake(effects, heard, audio.audio_byte_count);
    if heard.is_empty() {
        effects.report("Ignored wake: empty transcript");
    }
    !heard.is_empty()
}

fn wake(
    listener: &mut WakeListener,
    boundaries: Boundaries,
    (heard, audio): (&str, &HeardSegment),
) -> Boundary<()> {
    match held_match(listener, boundaries, (heard, audio)) {
        Some(matched) => open_for(listener.dry_run, boundaries, &matched),
        None => Ok(()),
    }
}

fn open_for(dry_run: bool, boundaries: Boundaries, matched: &WakeMatch) -> Boundary<()> {
    let dispatch = Dispatch {
        dry_run,
        matched,
        setting: Setting::Opening,
        sequence: None,
    };
    if !deliver(boundaries.effects, &dispatch)? {
        return Ok(());
    }
    open_and_hand(boundaries, matched)
}

fn open_and_hand(boundaries: Boundaries, matched: &WakeMatch) -> Boundary<()> {
    boundaries.seat.open_conversation()?;
    boundaries.seat.hand_to(&addressee_of(&matched.session, 0))
}

fn held_match(
    listener: &mut WakeListener,
    boundaries: Boundaries,
    (heard, audio): (&str, &HeardSegment),
) -> Option<WakeMatch> {
    let dry_run = listener.dry_run;
    with_addressing(listener, boundaries, |addressing, view| {
        let matched = addressing.wake_match(heard)?;
        holds_the_room(view, dry_run, &matched.session, audio).then_some(matched)
    })
}
