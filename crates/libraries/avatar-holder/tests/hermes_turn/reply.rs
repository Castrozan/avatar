use super::{case_name, cases, outcome_matches};
use avatar_holder::hermes_turn::final_reply;
use proptest::prelude::*;
use std::error::Error;

const FIXTURE: &str = include_str!("fixtures/hermes_turn_result.json");
const ANY_TEXT: &str = "[^\\x{85}\\x{2028}\\x{2029}]*";

#[test]
fn final_reply_matches_the_python_oracle() -> Result<(), Box<dyn Error>> {
    for case in cases(FIXTURE)? {
        let actual = final_reply(case["output"].as_str().unwrap_or_default());
        assert!(
            outcome_matches(&actual, &case["expected"]),
            "case {}",
            case_name(&case)
        );
    }
    Ok(())
}

fn result_line(text: &str) -> String {
    serde_json::json!({"type": "result", "text": text}).to_string()
}

proptest! {
    #[test]
    fn the_text_of_a_result_event_is_the_reply(text in ANY_TEXT) {
        prop_assert_eq!(final_reply(&result_line(&text)).ok(), Some(text));
    }

    #[test]
    fn the_last_result_event_wins(texts in prop::collection::vec(ANY_TEXT, 1..6)) {
        let output = texts.iter().map(|text| result_line(text)).collect::<Vec<_>>().join("\n");
        prop_assert_eq!(final_reply(&output).ok(), texts.last().cloned());
    }

    #[test]
    fn trailing_noise_does_not_hide_the_result(text in ANY_TEXT, noise in "[a-z ]{1,20}") {
        let output = format!("{}\n{}", result_line(&text), noise);
        prop_assert_eq!(final_reply(&output).ok(), Some(text));
    }
}
