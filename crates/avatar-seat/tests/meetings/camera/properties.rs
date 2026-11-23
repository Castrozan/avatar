use avatar_seat::meetings::camera::python_json;
use proptest::prelude::*;
use serde_json::Value;

use super::harness::Harness;
use super::store::HOME;

fn reported_after(sleeps_before_report: u32) -> Option<String> {
    let mut harness = Harness::new(HOME, &[]);
    let store = harness.store.clone();
    let landing =
        move |count: u32| (count == sleeps_before_report).then(|| store.publish_report("active"));
    harness.clock.on_sleep = Box::new(move |count| landing(count).unwrap_or_default());
    publish_if(sleeps_before_report == 0, &harness);
    harness.camera().reported_camera_state()
}

fn publish_if(due: bool, harness: &Harness) {
    if due {
        harness.store.publish_report("active");
    }
}

proptest! {
    #[test]
    fn the_report_is_seen_exactly_when_it_lands_inside_thirty_seconds(sleeps in 0_u32..140) {
        prop_assert_eq!(reported_after(sleeps).is_some(), sleeps < 120);
    }

    #[test]
    fn python_json_is_ascii_and_round_trips(text in any::<String>()) {
        let rendered = python_json(&text);
        prop_assert!(rendered.is_ascii());
        prop_assert_eq!(serde_json::from_str::<Value>(&rendered).ok(), Some(Value::String(text)));
    }
}
