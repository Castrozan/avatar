use avatar_mac::talk::LiveOpening;
use serde_json::json;

use super::commands::parsed;
use super::live::{Hooks, live_with};
use crate::fakes::TestResult;

const BRIEF: &str = "The nightly build is flaky; retrying twice was just decided.";

#[test]
fn a_live_opening_carries_its_brief() -> TestResult {
    let request = parsed(&["open", "--live", "--brief", BRIEF])?;
    assert_eq!(request.brief.as_deref(), Some(BRIEF));
    Ok(())
}

#[test]
fn a_brief_without_the_live_voice_is_refused_on_the_mac() {
    let refused = parsed(&["open", "--brief", BRIEF]);
    assert!(refused.is_err_and(|failure| failure.to_string().contains("--live")));
}

#[test]
fn the_live_voice_opened_on_the_mac_begins_with_the_brief() -> TestResult {
    let request = parsed(&["open", "--live", "--brief", BRIEF])?;
    let mut live = live_with(Some("p1"), true);
    live.open_live_conversation(&request, &mut Hooks::default())?;
    assert_eq!(live.boundary.begun[0].settings["brief"], json!(BRIEF));
    Ok(())
}
