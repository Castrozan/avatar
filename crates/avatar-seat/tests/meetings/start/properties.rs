use avatar_seat::local::SeatFailure;
use avatar_seat::meetings::readiness::Check;
use avatar_seat::meetings::start::{ReadinessWait, Target, wait_until_ready};
use proptest::prelude::*;
use serde_json::{Value, json};

use super::scripted::Fakes;

fn scenario_with(renderer: &[bool]) -> Value {
    json!({"runs": [], "audio": [true], "renderer": renderer, "camera": [true], "prepare_fails": null})
}

fn wait_for(timeout_seconds: u32) -> ReadinessWait {
    ReadinessWait {
        units: Target::Window.units(),
        checks: vec![Check::Renderer],
        timeout_seconds,
    }
}

fn sleeps(fakes: &Fakes) -> usize {
    let calls = fakes.log.borrow().clone();
    calls
        .iter()
        .filter(|call| call.as_str() == Some("S:0.25"))
        .count()
}

proptest! {
    #[test]
    fn a_service_that_never_becomes_ready_costs_four_polls_a_second(timeout in 0_u32..30) {
        let fakes = Fakes::from_scenario(&scenario_with(&[false]));
        let outcome = wait_until_ready(&fakes.edges(), &wait_for(timeout));
        prop_assert!(matches!(outcome, Err(SeatFailure::Timeout(_))));
        prop_assert_eq!(sleeps(&fakes), timeout as usize * 4);
    }

    #[test]
    fn readiness_after_some_polls_succeeds_exactly_inside_the_deadline(polls in 0_usize..40) {
        let mut renderer = vec![false; polls];
        renderer.push(true);
        let fakes = Fakes::from_scenario(&scenario_with(&renderer));
        let outcome = wait_until_ready(&fakes.edges(), &wait_for(5));
        prop_assert_eq!(outcome.is_ok(), polls < 20);
        prop_assert_eq!(sleeps(&fakes), polls.min(20));
    }
}
