use avatar_conversation::talk::run;
use serde_json::{Value, json};

use super::fake::World;

const BRIEF: &str = "The nightly build is flaky; retrying twice was just decided.";
const PANE: &str = "wW:pPT";
const REFUSAL: &str = "avatar-talk open --brief needs --live: only the live voice takes a brief\n";

fn holder(host: Option<&str>) -> String {
    json!({"pane": PANE, "servant": "Suzuka", "heard_through": 0, "host": host}).to_string()
}

fn opened(words: &[&str], host: Option<&str>) -> Value {
    let scenario = json!({
        "entry": "talk", "argv": words, "ignore_streams": false,
        "env": {"HERDR_PANE_ID": PANE, "AVATAR_AGENT_HOST": "chise"},
        "files": {"addressee.json": holder(host)},
    });
    let argv: Vec<String> = words.iter().map(|word| (*word).to_owned()).collect();
    let mut world = World::new(&scenario);
    let exit = run(&mut world, &argv);
    world.outcome(exit, None, &Value::Null)
}

fn calls_of(outcome: &Value) -> Vec<&str> {
    let log = outcome["log"].as_array().map_or(&[][..], Vec::as_slice);
    log.iter()
        .filter_map(|entry| entry["call"].as_str())
        .collect()
}

fn live_settings(outcome: &Value) -> Value {
    let log = outcome["log"].as_array().map_or(&[][..], Vec::as_slice);
    let begun = log.iter().find(|entry| entry["call"] == "begin_listening");
    begun.map_or(Value::Null, |entry| entry["settings"].clone())
}

#[test]
fn the_brief_reaches_the_live_settings_next_to_the_seat() {
    let outcome = opened(&["open", "--live", "--brief", BRIEF], None);
    assert_eq!(outcome["exit"], 0, "{outcome}");
    assert_eq!(
        live_settings(&outcome),
        json!({"seat": "room", "brief": BRIEF})
    );
}

#[test]
fn the_brief_reaches_the_live_settings_when_a_remote_agent_is_named() {
    let words = ["open", "--live", "--agent", "mac:wW:pPT", "--brief", BRIEF];
    let outcome = opened(&words, Some("mac"));
    assert_eq!(outcome["exit"], 0, "{outcome}");
    assert!(calls_of(&outcome).contains(&"hand_to_remote_agent"));
    assert_eq!(
        live_settings(&outcome),
        json!({"seat": "room", "brief": BRIEF})
    );
}

#[test]
fn a_live_open_without_a_brief_keeps_the_settings_it_always_had() {
    let outcome = opened(&["open", "--live"], None);
    assert_eq!(outcome["exit"], 0, "{outcome}");
    assert_eq!(live_settings(&outcome), json!({"seat": "room"}));
}

#[test]
fn a_brief_without_the_live_voice_is_refused_before_anything_starts() {
    let outcome = opened(&["open", "--brief", BRIEF], None);
    assert_eq!(outcome["exit"], 1);
    assert_eq!(outcome["stderr"], REFUSAL);
    assert_eq!(calls_of(&outcome), ["report_talk"]);
}
