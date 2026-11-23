mod camera;
mod hold;
mod keepers;
mod world;

use std::cell::RefCell;

use avatar_seat::holder::{MeetingSeat, Reporter, SeatKeeper};
use camera::ScriptedCamera;
use serde::Deserialize;
use serde_json::Value;

#[derive(Default)]
pub(crate) struct Reports(pub(crate) RefCell<Vec<String>>);

impl Reporter for Reports {
    fn report(&self, message: &str) {
        self.0.borrow_mut().push(message.to_owned());
    }
}

#[derive(Deserialize)]
struct Looks {
    #[serde(rename = "looks")]
    seats: Vec<Value>,
    camera: Value,
    end: bool,
    results: Vec<bool>,
    calls: Vec<String>,
    reports: Vec<String>,
    owed: bool,
    absent: u32,
}

fn look_once(seat: &mut impl SeatKeeper, look: &Value) -> bool {
    seat.look(Some(look).filter(|value| !value.is_null()))
}

fn replay(scenario: &Looks, reports: &Reports) -> (Vec<bool>, Vec<String>, (bool, u32)) {
    let mut seat = MeetingSeat::new(ScriptedCamera::new(&scenario.camera), reports);
    let results = scenario
        .seats
        .iter()
        .map(|look| look_once(&mut seat, look))
        .collect();
    scenario.end.then(|| seat.end());
    let state = (seat.owed, seat.looks_without_a_meeting);
    (results, seat.camera.calls(), state)
}

fn check_scenario(scenario: &Looks) {
    let reports = Reports::default();
    let (results, calls, state) = replay(scenario, &reports);
    assert_eq!(
        (results, calls),
        (scenario.results.clone(), scenario.calls.clone())
    );
    assert_eq!(reports.0.borrow().clone(), scenario.reports);
    assert_eq!(state, (scenario.owed, scenario.absent));
}

#[test]
fn every_meeting_seat_scenario_behaves_as_the_python_did() {
    let scenarios: Vec<Looks> =
        serde_json::from_str(include_str!("fixtures/looks.json")).unwrap_or_default();
    assert!(scenarios.len() >= 20);
    scenarios.iter().for_each(check_scenario);
}
