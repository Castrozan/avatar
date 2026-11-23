use avatar_floor::claim::held_by_another_room;
use serde_json::json;
use std::error::Error;

fn record(room: &str, claimed_at: f64) -> Result<String, serde_json::Error> {
    serde_json::to_string(&json!({"room": room, "claimed_at": claimed_at}))
}

#[test]
fn a_recent_claim_by_another_room_holds() -> Result<(), Box<dyn Error>> {
    assert!(held_by_another_room(
        &record("alpha", 100.0)?,
        "beta",
        105.0
    ));
    Ok(())
}

#[test]
fn a_claim_just_inside_ten_seconds_holds() -> Result<(), Box<dyn Error>> {
    assert!(held_by_another_room(
        &record("alpha", 100.0)?,
        "beta",
        109.5
    ));
    Ok(())
}

#[test]
fn a_claim_exactly_ten_seconds_old_has_expired() -> Result<(), Box<dyn Error>> {
    assert!(!held_by_another_room(
        &record("alpha", 100.0)?,
        "beta",
        110.0
    ));
    Ok(())
}

#[test]
fn the_same_room_never_holds_against_itself() -> Result<(), Box<dyn Error>> {
    assert!(!held_by_another_room(
        &record("alpha", 100.0)?,
        "alpha",
        101.0
    ));
    Ok(())
}

#[test]
fn an_empty_holder_holds_nothing() -> Result<(), Box<dyn Error>> {
    assert!(!held_by_another_room(&record("", 100.0)?, "beta", 101.0));
    Ok(())
}

#[test]
fn a_record_without_a_time_counts_as_claimed_at_the_epoch() -> Result<(), Box<dyn Error>> {
    let untimed = serde_json::to_string(&json!({"room": "alpha"}))?;
    assert!(held_by_another_room(&untimed, "beta", 5.0));
    assert!(!held_by_another_room(&untimed, "beta", 10.0));
    Ok(())
}

#[test]
fn a_claim_stamped_in_the_future_still_holds() -> Result<(), Box<dyn Error>> {
    assert!(held_by_another_room(&record("alpha", 100.0)?, "beta", 90.0));
    Ok(())
}
