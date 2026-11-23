use avatar_conversation::voice_turns::{
    SpeechRequest, acknowledgement, speech_command, speech_moments,
};
use proptest::prelude::*;
use serde_json::json;

proptest! {
    #[test]
    fn an_acknowledgement_is_always_a_speak_ack_object(output in "(?s).{0,200}") {
        if let Some(response) = acknowledgement(&output) {
            prop_assert_eq!(response["type"].as_str(), Some("speakAck"));
        }
    }

    #[test]
    fn every_moment_is_the_timing_value_over_a_thousand(milliseconds in any::<u32>()) {
        let response = json!({"timing": {"synthesized": milliseconds}});
        let moments = speech_moments(&response);
        prop_assert_eq!(moments.len(), 1);
        prop_assert_eq!(moments[0].1.to_bits(), (f64::from(milliseconds) / 1000.0).to_bits());
    }

    #[test]
    fn the_voice_is_passed_only_when_it_is_not_empty(voice in proptest::option::of(".{0,12}")) {
        let request = SpeechRequest { text: "oi".into(), emotion: "neutral".into(), voice: voice.clone() };
        let expected = 5 + usize::from(voice.as_deref().is_some_and(|value| !value.is_empty()));
        prop_assert_eq!(speech_command(&request, "speaker").len(), expected);
    }
}
