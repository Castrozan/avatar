use super::{case_name, cases};
use avatar_holder::hermes_turn::{STAY_SILENT_REPLY, spoken_turn_prompt};
use proptest::prelude::*;
use std::error::Error;

const FIXTURE: &str = include_str!("fixtures/hermes_turn_prompt.json");

#[test]
fn spoken_turn_prompt_matches_the_python_oracle() -> Result<(), Box<dyn Error>> {
    for case in cases(FIXTURE)? {
        let heard = case["heard"].as_str().unwrap_or_default();
        assert_eq!(
            spoken_turn_prompt(heard),
            case["prompt"].as_str().unwrap_or_default(),
            "case {}",
            case_name(&case)
        );
    }
    Ok(())
}

proptest! {
    #[test]
    fn the_prompt_ends_with_the_heard_words_and_keeps_the_silence_rule(heard in ".*") {
        let prompt = spoken_turn_prompt(&heard);
        let suffix = format!("Lucas: {heard}");
        prop_assert!(prompt.ends_with(&suffix));
        prop_assert!(prompt.contains(STAY_SILENT_REPLY));
    }
}
