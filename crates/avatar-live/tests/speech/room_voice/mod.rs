mod replay;

use serde_json::{Value, json};

use self::replay::Replay;
use crate::support::{Outcome, array, same, text, try_cases};

const FIXTURE: &str = include_str!("fixtures/room_voice.json");

fn voice_matches(case: &Value, replay: &Replay, generation: usize, voice: &Value) {
    same(
        &json!(replay.deliveries[generation]),
        &voice["delivered"],
        case,
    );
    same(&json!(replay.audio[generation]), &voice["heard"], case);
}

fn check(case: &Value) -> Outcome {
    let mut replay = Replay::new();
    array(&case["ops"])
        .iter()
        .try_for_each(|step| replay.apply(step))?;
    same(&json!(replay.trace), &case["trace"], case);
    assert_eq!(replay.state.speaking_voice(), text(&case["speaking_voice"]));
    let voices = array(&case["voices"]).iter().enumerate();
    voices.for_each(|(generation, voice)| voice_matches(case, &replay, generation, voice));
    Ok(())
}

#[test]
fn every_recorded_python_room_voice_scenario_is_reproduced() -> Outcome {
    try_cases(FIXTURE, 10, check)
}
