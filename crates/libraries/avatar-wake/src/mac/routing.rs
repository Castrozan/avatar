use avatar_floor::addressee::Addressee;
use avatar_floor::quiet::{Entry, ListenerPurpose};
use avatar_holder::floor_holder::passes;

use super::conversation::{CONVERSATION_READ_REUSE_SECONDS, MacConversation, RemoteRead};
use super::remote::{hand_to_the_avatar_host, read_entries, remote_addressee, remote_purpose};
use crate::listener::Boundary;

fn lately_listening_here(seat: &MacConversation) -> bool {
    let now = seat.edges.clock();
    let mut throttle = seat.lately_listening_here.borrow_mut();
    passes(&mut throttle, now, || seat.edges.listening_here())
}

fn reuses_the_last_read(remote: &RemoteRead, now: f64) -> bool {
    remote
        .read_at
        .is_some_and(|read| now - read < CONVERSATION_READ_REUSE_SECONDS)
}

fn remote_entries(seat: &MacConversation) -> Boundary<Vec<Entry>> {
    let mut remote = seat.remote.borrow_mut();
    let now = seat.edges.clock();
    if reuses_the_last_read(&remote, now) {
        return Ok(remote.entries.clone());
    }
    remote.read_at = Some(now);
    remote.entries = read_entries(seat.edges)?;
    Ok(remote.entries.clone())
}

fn remote_when_listening(seat: &MacConversation) -> Boundary<Vec<Entry>> {
    if seat.edges.seat_listening() {
        return remote_entries(seat);
    }
    *seat.remote.borrow_mut() = RemoteRead::default();
    Ok(Vec::new())
}

pub(super) fn transcript_entries(seat: &MacConversation) -> Boundary<Vec<Entry>> {
    if lately_listening_here(seat) {
        return seat.here.transcript_entries();
    }
    remote_when_listening(seat)
}

pub(super) fn speech_in_progress(seat: &MacConversation) -> bool {
    !lately_listening_here(seat) || seat.here.speech_in_progress()
}

pub(super) fn listening_purpose(seat: &MacConversation) -> Boundary<Option<ListenerPurpose>> {
    if seat.edges.listening_here() {
        return seat.here.listening_purpose();
    }
    if !seat.edges.seat_listening() {
        return Ok(None);
    }
    remote_purpose(seat.edges)
}

pub(super) fn addressee(seat: &MacConversation) -> Boundary<Option<Addressee>> {
    if seat.edges.listening_here() {
        return seat.here.addressee();
    }
    if !seat.edges.seat_listening() {
        return Ok(None);
    }
    remote_addressee(seat.edges)
}

pub(super) fn quiet_seconds(seat: &MacConversation) -> f64 {
    if seat.edges.listening_here() {
        return seat.here.quiet_seconds();
    }
    0.0
}

pub(super) fn hand_to(seat: &MacConversation, addressee: &Addressee) -> Boundary<()> {
    if seat.edges.listening_here() {
        return seat.here.hand_to(addressee);
    }
    hand_to_the_avatar_host(seat.edges, addressee)
}
