use avatar_conversation::utterances::{
    HeardEntry, OpenAcrossRestarts, consumed_utterance, last_heard, settle_seconds_after,
};
use proptest::prelude::*;

fn entries() -> impl Strategy<Value = Vec<HeardEntry>> {
    proptest::collection::vec("[a-z ]{0,8}", 1..6).prop_map(numbered_entries)
}

fn numbered_entries(texts: Vec<String>) -> Vec<HeardEntry> {
    texts
        .into_iter()
        .enumerate()
        .map(|(index, text)| HeardEntry {
            sequence: index as u64 + 1,
            text,
            turn_finished: index % 2 == 0,
        })
        .collect()
}

proptest! {
    #[test]
    fn last_heard_is_the_maximum_of_cursor_and_addressee(heard in any::<u64>(), through in proptest::option::of(any::<u64>())) {
        let expected = through.map_or(heard, |value| heard.max(value));
        prop_assert_eq!(last_heard(heard, through), expected);
        prop_assert!(last_heard(heard, through) >= heard);
    }

    #[test]
    fn the_settle_wait_is_one_of_the_two_python_constants(finished in any::<bool>()) {
        let expected = if finished { 0.5f64 } else { 2.5f64 };
        prop_assert_eq!(settle_seconds_after(finished).to_bits(), expected.to_bits());
    }

    #[test]
    fn the_consumed_utterance_ends_at_the_last_sequence(list in entries()) {
        let consumed = consumed_utterance(&list);
        prop_assert!(consumed.is_some());
        let last = list.last().map(|entry| entry.sequence);
        prop_assert_eq!(consumed.map(|value| value.last_sequence), last);
    }

    #[test]
    fn an_open_conversation_always_passes(events in proptest::collection::vec((any::<bool>(), 0.0f64..1000.0), 1..30)) {
        let mut restart = OpenAcrossRestarts::new(20.0);
        for (open, now) in events {
            let passes = restart.passes(open, now);
            if open {
                prop_assert!(passes);
            }
        }
    }

    #[test]
    fn the_grace_window_never_outlasts_its_length(now in 0.0f64..1000.0, later in 0.0f64..1000.0) {
        let mut restart = OpenAcrossRestarts::new(20.0);
        restart.passes(false, now);
        prop_assert_eq!(restart.passes(false, now + later), (now + later - now) < 20.0);
    }
}
