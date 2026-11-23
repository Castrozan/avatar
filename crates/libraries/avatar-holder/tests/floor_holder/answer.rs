use super::cases;
use crate::room::SharedRoom;
use crate::voice::SharedVoice;
use avatar_holder::default_holder::DefaultHolder;
use avatar_holder::floor_holder::{FloorHolder, answer_for};
use avatar_holder::hermes_turn::{FAILED_TURN_APOLOGY, HermesTurnError};
use proptest::prelude::*;
use serde_json::json;
use std::error::Error;

const FIXTURE: &str = include_str!("fixtures/voice_holder_answer.json");
const TURN_EXITED: &str = "The Hermes turn exited 3";

type Scenario = (
    &'static str,
    Result<&'static str, &'static str>,
    bool,
    Option<&'static str>,
);

const SCENARIOS: [Scenario; 7] = [
    ("worth speaking and addressed", Ok("Oi."), true, None),
    ("silence not spoken", Ok("SILENCIO"), true, None),
    ("hermes failure apologised", Err(TURN_EXITED), true, None),
    ("addressed away after reply", Ok("Oi."), false, None),
    ("empty reply", Ok(""), true, None),
    ("failure when addressed away", Err(TURN_EXITED), false, None),
    ("speech failure reported", Ok("Oi."), true, Some("boom")),
];

fn scenario_named(name: &str) -> Option<&'static Scenario> {
    SCENARIOS.iter().find(|scenario| scenario.0 == name)
}

fn room_addressed(addressed: bool) -> SharedRoom {
    let room = SharedRoom::default();
    room.0.borrow_mut().addressed_now = addressed;
    room
}

fn voice_for(scenario: &Scenario) -> SharedVoice {
    let voice = SharedVoice::default();
    voice.0.borrow_mut().replies =
        vec![scenario.1.map(str::to_owned).map_err(HermesTurnError::new)].into();
    voice.0.borrow_mut().speak_failure = scenario.3.map(str::to_owned);
    voice
}

fn run_answer(scenario: &Scenario) -> (Vec<String>, String) {
    let voice = voice_for(scenario);
    FloorHolder::new(
        DefaultHolder::new("silver", None),
        room_addressed(scenario.2),
        voice.clone(),
    )
    .answer("olá");
    spoken_and_reported(&voice)
}

fn spoken_and_reported(voice: &SharedVoice) -> (Vec<String>, String) {
    let state = voice.0.borrow();
    let reported = state
        .reports
        .iter()
        .flat_map(|report| [report.as_str(), "\n"])
        .collect();
    (state.spoken.clone(), reported)
}

fn answer_matches(case: &serde_json::Value) -> Result<bool, Box<dyn Error>> {
    let scenario = scenario_named(case["name"].as_str().unwrap_or_default());
    let (spoken, stderr) = run_answer(scenario.ok_or("no scenario for this case")?);
    Ok(json!(spoken) == case["spoken"] && json!(stderr) == case["stderr"])
}

#[test]
fn answering_matches_the_python_oracle() -> Result<(), Box<dyn Error>> {
    for case in cases(FIXTURE)? {
        assert!(answer_matches(&case)?, "case {}", case["name"]);
    }
    Ok(())
}

proptest! {
    #[test]
    fn a_failed_turn_is_always_reported_and_answered_with_the_apology(message in "[ -~]{1,40}") {
        let answer = answer_for(Err(HermesTurnError::new(message.clone())));
        prop_assert_eq!(answer.report, Some(message));
        prop_assert_eq!(answer.reply, FAILED_TURN_APOLOGY);
    }

    #[test]
    fn a_successful_turn_is_answered_with_its_text(text in ".*") {
        let answer = answer_for(Ok(text.clone()));
        prop_assert_eq!(answer.report, None);
        prop_assert_eq!(answer.reply, text);
    }
}
