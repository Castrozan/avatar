use std::cell::RefCell;

use avatar_routing::names::session::Session;

use super::boundaries::Boundaries;
use super::machine::WakeListener;
use crate::addressing::VoiceAddressing;

pub(super) const FLEET_REFRESH_SECONDS: f64 = 60.0;

#[derive(Debug, Default)]
pub(super) struct SessionCache {
    sessions: Vec<Session>,
    read_at: Option<f64>,
}

pub(super) struct View<'a> {
    pub(super) boundaries: Boundaries<'a>,
    pub(super) cache: &'a RefCell<SessionCache>,
}

impl View<'_> {
    pub(super) fn refresh(&self) -> Vec<Session> {
        let sessions = self.boundaries.effects.sessions();
        let mut cache = self.cache.borrow_mut();
        cache.sessions.clone_from(&sessions);
        cache.read_at = Some(self.boundaries.effects.now());
        sessions
    }

    fn stale(&self) -> bool {
        let read_at = self.cache.borrow().read_at;
        read_at.is_none_or(|read| self.boundaries.effects.now() - read >= FLEET_REFRESH_SECONDS)
    }

    pub(super) fn known(&self) -> Vec<Session> {
        refresh_when_stale(self);
        self.cache.borrow().sessions.clone()
    }
}

fn refresh_when_stale(view: &View) {
    if view.stale() {
        view.refresh();
    }
}

pub(super) fn armed(cache: &RefCell<SessionCache>, boundaries: Boundaries) -> bool {
    let view = View { boundaries, cache };
    !view.known().is_empty() && !boundaries.seat.conversation_listening()
}

pub(super) fn with_addressing<Found>(
    listener: &mut WakeListener,
    boundaries: Boundaries,
    decide: impl FnOnce(&mut VoiceAddressing, &View) -> Found,
) -> Found {
    let view = View {
        boundaries,
        cache: &listener.cache,
    };
    let state = std::mem::take(&mut listener.addressing);
    let mut addressing = VoiceAddressing::resume(&view, &view, state);
    let found = decide(&mut addressing, &view);
    listener.addressing = addressing.into_state();
    found
}

mod view;
