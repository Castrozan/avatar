use avatar_transcript::echo::{EchoFilter, normalized, resembles};
use proptest::prelude::*;

proptest! {
    #[test]
    fn normalization_is_idempotent(text in ".{0,80}") {
        let once = normalized(&text);
        prop_assert_eq!(normalized(&once), once);
    }

    #[test]
    fn normalized_text_has_single_separators(text in ".{0,80}") {
        let once = normalized(&text);
        prop_assert!(!once.starts_with(' ') && !once.ends_with(' ') && !once.contains("  "));
    }

    #[test]
    fn every_text_resembles_itself(text in ".{0,80}") {
        prop_assert!(resembles(&text, &text));
    }

    #[test]
    fn a_source_never_echoes_its_own_repeats(
        utterances in prop::collection::vec((".{0,40}", 0u16..1000), 1..8),
    ) {
        let mut filter = EchoFilter::default();
        for (text, now) in utterances {
            prop_assert!(!filter.is_echo("human", &text, f64::from(now)));
        }
    }

    #[test]
    fn another_source_repeating_within_the_window_is_an_echo(text in ".{0,40}", gap in 0u16..=15) {
        let mut filter = EchoFilter::default();
        prop_assert!(!filter.is_echo("meeting", &text, 0.0));
        prop_assert!(filter.is_echo("human", &text, f64::from(gap)));
    }

    #[test]
    fn another_source_after_the_window_is_not_an_echo(text in ".{0,40}", gap in 16u16..5000) {
        let mut filter = EchoFilter::default();
        prop_assert!(!filter.is_echo("meeting", &text, 0.0));
        prop_assert!(!filter.is_echo("human", &text, f64::from(gap)));
    }
}
