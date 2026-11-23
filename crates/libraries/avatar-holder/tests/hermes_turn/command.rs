use super::{case_name, cases};
use avatar_holder::hermes_turn::hermes_command;
use proptest::prelude::*;
use std::error::Error;

const FIXTURE: &str = include_str!("fixtures/hermes_turn_command.json");

#[test]
fn hermes_command_matches_the_python_oracle() -> Result<(), Box<dyn Error>> {
    for case in cases(FIXTURE)? {
        let profile = case["profile"].as_str().unwrap_or_default();
        let prompt = case["prompt"].as_str().unwrap_or_default();
        let actual = hermes_command(profile, prompt, "2026-11-23");
        let expected: Vec<String> = serde_json::from_value(case["argv"].clone())?;
        assert_eq!(actual, expected, "case {}", case_name(&case));
    }
    Ok(())
}

proptest! {
    #[test]
    fn the_profile_and_prompt_land_in_their_flags(
        profile in "[a-z -]{0,12}",
        prompt in ".*",
        day in "[0-9]{4}-[0-9]{2}-[0-9]{2}",
    ) {
        let command = hermes_command(&profile, &prompt, &day);
        prop_assert_eq!(command.len(), 16);
        prop_assert_eq!(&command[2], &profile);
        prop_assert_eq!(&command[5], &prompt);
        prop_assert_eq!(&command[8], &format!("voice-call-{day}"));
    }
}
