use crate::fakes::{addressee, session};
use avatar_floor::floor::{first_claim, holder_has_a_live_session, may_speak};

#[test]
fn no_holder_has_no_live_session() {
    assert!(!holder_has_a_live_session(None, &[]));
}

#[test]
fn a_hermes_holder_is_live_without_a_session() {
    let holder = addressee("hermes:x", "s", 0, None);
    assert!(holder_has_a_live_session(Some(&holder), &[]));
}

#[test]
fn a_holder_with_a_session_on_its_pane_is_live() {
    let holder = addressee("p1", "s", 0, None);
    assert!(holder_has_a_live_session(
        Some(&holder),
        &[session("p1", "s", None)]
    ));
}

#[test]
fn a_holder_without_a_session_on_its_pane_is_dead() {
    let holder = addressee("p1", "s", 0, None);
    assert!(!holder_has_a_live_session(
        Some(&holder),
        &[session("p2", "s", None)]
    ));
}

#[test]
fn first_claim_needs_a_session_on_the_pane() {
    assert_eq!(first_claim("p1", &[session("p2", "s", None)], None), None);
}

#[test]
fn first_claim_starts_at_zero_without_a_holder() {
    let claimed = first_claim("p1", &[session("p1", "s1", None)], None);
    assert_eq!(claimed, Some(addressee("p1", "s1", 0, None)));
}

#[test]
fn first_claim_carries_the_holder_heard_through() {
    let holder = addressee("p9", "s9", 42, None);
    let claimed = first_claim("p1", &[session("p1", "s1", None)], Some(&holder));
    assert_eq!(claimed, Some(addressee("p1", "s1", 42, None)));
}

#[test]
fn anyone_may_speak_without_a_holder() {
    assert!(may_speak(None, "p1"));
}

#[test]
fn the_holder_may_speak() {
    let holder = addressee("p1", "s", 0, None);
    assert!(may_speak(Some(&holder), "p1"));
}

#[test]
fn another_pane_may_not_speak_over_the_holder() {
    let holder = addressee("p2", "s", 0, None);
    assert!(!may_speak(Some(&holder), "p1"));
}
