use avatar_wake::peer::listing::{listing_of, sessions_everywhere};
use serde_json::Value;
use std::error::Error;

use super::support::{cases, seat_of, session_json, sessions_list};

const FIXTURE: &str = include_str!("fixtures/peer.json");

#[test]
fn a_listing_of_this_machine_names_its_panes_and_seat() -> Result<(), Box<dyn Error>> {
    cases(FIXTURE, "listing_of")?
        .iter()
        .for_each(check_listing_of_case);
    Ok(())
}

fn check_listing_of_case(case: &Value) {
    let sessions = sessions_list(&case["sessions"]);
    let rendered = listing_of(&sessions, seat_of(&case["conversation_seat"]));
    assert_eq!(
        serde_json::from_str::<Value>(&rendered).ok(),
        Some(case["expected"].clone())
    );
}

#[test]
fn every_servant_is_heard_once_and_the_local_session_wins() -> Result<(), Box<dyn Error>> {
    cases(FIXTURE, "everywhere")?
        .iter()
        .for_each(check_everywhere_case);
    Ok(())
}

fn check_everywhere_case(case: &Value) {
    let merged = sessions_everywhere(
        &sessions_list(&case["local"]),
        &sessions_list(&case["remote"]),
    );
    let rendered: Vec<Value> = merged.iter().map(session_json).collect();
    assert_eq!(Value::Array(rendered), case["expected"]);
}
