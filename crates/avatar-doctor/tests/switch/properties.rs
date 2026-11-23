use avatar_doctor::switch::moment::{Moments, Switch};
use avatar_doctor::switch::record::logged_records;
use avatar_doctor::switch::report::summary::nearest_rank;
use avatar_doctor::switch::store::history::{KEPT_SWITCHES, merged};
use proptest::collection::vec;
use proptest::prelude::*;

fn switch_at(switched_at: f64) -> Switch {
    Switch {
        switched_at,
        servant: None,
        pane: None,
        sequence: None,
        delivered: None,
        moments: Moments::unrecorded(),
    }
}

fn count_of(length: usize) -> f64 {
    f64::from(u32::try_from(length).unwrap_or(u32::MAX))
}

proptest! {
    #[test]
    fn a_nearest_rank_splits_the_measurements_at_its_rank(values in vec(0.0_f64..60.0, 1..40), percentile in 1_u32..=100) {
        let rank = nearest_rank(&values, percentile).ok_or_else(|| TestCaseError::fail("empty"))?;
        let share = f64::from(percentile) / 100.0 * count_of(values.len());
        prop_assert!(values.contains(&rank));
        prop_assert!(count_of(values.iter().filter(|value| **value <= rank).count()) >= share);
        prop_assert!(count_of(values.iter().filter(|value| **value < rank).count()) < share);
    }

    #[test]
    fn any_log_text_parses_without_panicking_and_stays_in_time_order(lines in vec(".*", 0..30)) {
        let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
        if let Ok(logged) = logged_records(refs) {
            prop_assert!(logged.windows(2).all(|pair| pair[0].at <= pair[1].at));
        }
    }

    #[test]
    fn merged_switches_are_unique_ordered_and_capped(
        remembered in vec(0_u32..5000, 0..1200),
        fresh in vec(0_u32..5000, 0..50),
    ) {
        let history: Vec<Switch> = remembered.iter().map(|moment| switch_at(f64::from(*moment))).collect();
        let newer: Vec<Switch> = fresh.iter().map(|moment| switch_at(f64::from(*moment))).collect();
        let combined = merged(history, newer);
        prop_assert!(combined.len() <= KEPT_SWITCHES);
        prop_assert!(combined.windows(2).all(|pair| pair[0].switched_at < pair[1].switched_at));
    }
}
