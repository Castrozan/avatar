use avatar_seat::local::SeatFailure;
use avatar_seat::meetings::start::{
    BringUp, StartEdges, bring_up, service_failed, start, wait_until_ready,
};
use avatar_seat::meetings::virtual_meeting::{require_fresh_runtime, unit_states};
use serde_json::{Value, json};

use super::arguments::{checks_of, seat_of, strings, target_of, waiting};
use super::scripted::Fakes;
use crate::fakes::failures::assert_failure;
use crate::fakes::processes::{Log, assert_calls};
use crate::fakes::seat_failures::seat_type;

const STATE_QUERY_SECONDS: u64 = 10;

fn record(log: &Log, value: &Value) {
    log.borrow_mut()
        .push(json!({"kind": "result", "value": value}));
}

fn bringing_up(args: &Value) -> BringUp {
    BringUp {
        target: target_of(args),
        checks: checks_of(&args["checks"]),
    }
}

fn states(args: &Value, edges: &StartEdges<'_>, log: &Log) -> Result<(), SeatFailure> {
    let found = unit_states(
        edges.processes,
        &strings(&args["units"]),
        STATE_QUERY_SECONDS,
    );
    found.map(|states| record(log, &json!(states)))
}

fn failed(args: &Value, edges: &StartEdges<'_>, log: &Log) -> Result<(), SeatFailure> {
    let found = service_failed(edges, &strings(&args["units"]));
    found.map(|failed| record(log, &json!(failed)))
}

fn fresh(args: &Value, edges: &StartEdges<'_>) -> Result<(), SeatFailure> {
    let units = strings(&args["units"]);
    require_fresh_runtime(
        edges.processes,
        edges.readiness,
        &units,
        STATE_QUERY_SECONDS,
    )
}

fn query(kind: &str, args: &Value, edges: &StartEdges<'_>, log: &Log) -> Result<(), SeatFailure> {
    match kind {
        "failed" => failed(args, edges, log),
        "states" => states(args, edges, log),
        _ => fresh(args, edges),
    }
}

fn operate(scenario: &Value, edges: &StartEdges<'_>, log: &Log) -> Result<(), SeatFailure> {
    let args = &scenario["args"];
    match scenario["kind"].as_str().unwrap_or_default() {
        "main" => start(edges, seat_of(args), args["virtual_only"] == true),
        "bring_up" => bring_up(edges, &bringing_up(args)),
        "wait" => wait_until_ready(edges, &waiting(args)),
        kind => query(kind, args, edges, log),
    }
}

fn assert_outcome(scenario: &Value, fakes: &Fakes, outcome: Result<(), SeatFailure>) {
    let failure = outcome
        .err()
        .map(|error| (seat_type(&error), error.to_string()));
    assert_failure(failure, &scenario["error"]);
    assert_eq!(
        fakes.output.text(),
        scenario["stdout"].as_str().unwrap_or_default()
    );
    assert_calls(&fakes.log, &scenario["calls"]);
}

pub(crate) fn replay(scenario: &Value) {
    let fakes = Fakes::from_scenario(scenario);
    let outcome = operate(scenario, &fakes.edges(), &fakes.log);
    assert_outcome(scenario, &fakes, outcome);
}
