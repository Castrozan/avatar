use avatar_holder::floor_holder::{OPEN_CHECK_SECONDS, ThrottledCheck, passes};
use proptest::prelude::*;

#[test]
fn the_first_check_runs_at_once_and_its_answer_holds_for_the_interval() {
    let mut throttle = ThrottledCheck::starting_at(10.0, OPEN_CHECK_SECONDS);
    assert!(!passes(&mut throttle, 10.0, || false));
    assert!(!passes(&mut throttle, 12.0, || true));
    assert!(passes(&mut throttle, 15.0, || true));
}

proptest! {
    #[test]
    fn checks_are_never_closer_than_the_interval(steps in prop::collection::vec(0.0f64..3.0, 1..200)) {
        let mut clock = 0.0;
        let mut throttle = ThrottledCheck::starting_at(clock, OPEN_CHECK_SECONDS);
        let mut check_times = Vec::new();
        for step in steps {
            clock += step;
            passes(&mut throttle, clock, || {
                check_times.push(clock);
                true
            });
        }
        for pair in check_times.windows(2) {
            prop_assert!(pair[1] - pair[0] >= OPEN_CHECK_SECONDS);
        }
    }
}
