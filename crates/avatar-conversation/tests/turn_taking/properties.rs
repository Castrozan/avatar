use avatar_conversation::turn_taking::{
    TurnObservation, WithheldReason, outcome_stops_reply, queued_behind_a_stopped_reply,
    withheld_reason,
};
use proptest::prelude::*;

proptest! {
    #[test]
    fn a_queued_reply_outranks_every_other_reason(speaking in any::<bool>(), unheard in any::<bool>()) {
        let value = TurnObservation { queued_behind_stopped_reply: true, speech_in_progress: speaking, unheard_words: unheard };
        prop_assert_eq!(withheld_reason(value), Some(WithheldReason::Queued));
    }

    #[test]
    fn speech_in_progress_outranks_unheard_words(unheard in any::<bool>()) {
        let value = TurnObservation { queued_behind_stopped_reply: false, speech_in_progress: true, unheard_words: unheard };
        prop_assert_eq!(withheld_reason(value), Some(WithheldReason::Speaking));
    }

    #[test]
    fn a_quiet_turn_is_withheld_only_for_unheard_words(unheard in any::<bool>()) {
        let value = TurnObservation { queued_behind_stopped_reply: false, speech_in_progress: false, unheard_words: unheard };
        prop_assert_eq!(withheld_reason(value), unheard.then_some(WithheldReason::Unheard));
    }

    #[test]
    fn the_reaction_window_is_one_second(age in 0.0f64..100.0) {
        prop_assert_eq!(queued_behind_a_stopped_reply(Some(age)), age < 1.0);
    }

    #[test]
    fn only_the_two_stopped_statuses_remember_a_reply(status in ".*") {
        let stops = matches!(status.as_str(), "interrupted" | "cancelled");
        prop_assert_eq!(outcome_stops_reply(Some(&status)), stops);
    }
}
