use super::{case_name, cases};
use avatar_holder::hermes_turn::{STAY_SILENT_REPLY, reply_worth_speaking};
use proptest::prelude::*;
use std::error::Error;

const FIXTURE: &str = include_str!("fixtures/hermes_turn_worth.json");

#[test]
fn reply_worth_speaking_matches_the_python_oracle() -> Result<(), Box<dyn Error>> {
    for case in cases(FIXTURE)? {
        let reply = case["reply"].as_str().unwrap_or_default();
        assert_eq!(
            reply_worth_speaking(reply),
            case["worth"].as_bool().unwrap_or_default(),
            "case {}",
            case_name(&case)
        );
    }
    Ok(())
}

proptest! {
    #[test]
    fn only_padding_around_silence_is_not_worth_speaking(
        leading in "[ .\n]*",
        trailing in "[ .\n]*",
    ) {
        let padded = format!("{leading}{STAY_SILENT_REPLY}{trailing}");
        prop_assert!(!reply_worth_speaking(&padded));
    }
}
