use avatar_live::session::DEFAULT_VOICE;
use avatar_live::voices::voice_for;
use proptest::prelude::*;
use serde_json::{Value, json};

use crate::support::{Outcome, run_cases, same};

const FIXTURE: &str = include_str!("fixtures/voice_choice.json");

fn check(case: &Value) {
    let voice = voice_for(case["character"].as_str(), case["table"].as_str());
    same(&json!(voice), &case["voice"], case);
}

#[test]
fn every_recorded_python_voice_choice_is_reproduced() -> Outcome {
    run_cases(FIXTURE, 35, check)
}

fn table_of(first: &str, second: &str) -> String {
    json!([{"name": "vivi", "cloudVoice": first}, {"name": "vita", "cloudVoice": second}])
        .to_string()
}

proptest! {
    #[test]
    fn the_voice_is_whatever_the_table_says_for_the_character(voice in "[a-f]{1,6}", other in "[a-f]{1,6}") {
        let table = table_of(&voice, &other);
        prop_assert_eq!(voice_for(Some("vivi"), Some(&table)), voice);
        prop_assert_eq!(voice_for(Some("nobody"), Some(&table)), DEFAULT_VOICE);
    }
}
