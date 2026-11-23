use avatar_listening::turn_detector::{TURN_BYTES, TurnTracker};
use proptest::prelude::*;

proptest! {
    #[test]
    fn the_judge_only_ever_sees_the_newest_model_window(
        segments in proptest::collection::vec(proptest::collection::vec(any::<u8>(), 0..3000), 1..40),
    ) {
        let mut tracker = TurnTracker::new(|_: &[u8]| false);
        let mut total = 0_usize;
        for segment in &segments {
            total += segment.len();
            tracker.add(segment);
        }
        prop_assert!(tracker.pending_audio().len() <= TURN_BYTES);
        prop_assert_eq!(tracker.pending_audio().len(), total.min(TURN_BYTES));
    }

    #[test]
    fn a_finished_turn_leaves_nothing_pending(
        segments in proptest::collection::vec(proptest::collection::vec(any::<u8>(), 0..500), 1..20),
    ) {
        let mut tracker = TurnTracker::new(|_: &[u8]| true);
        for segment in &segments {
            prop_assert!(tracker.add(segment));
            prop_assert!(tracker.pending_audio().is_empty());
        }
    }
}
