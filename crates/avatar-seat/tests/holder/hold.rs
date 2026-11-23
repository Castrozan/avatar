use avatar_seat::holder::{HoldEdges, hold_seat};
use avatar_seat::local::LocalSeat;
use serde_json::{Value, json};

use super::Reports;
use super::world::{Hold, World, config};

pub(crate) fn push_collapsing(result: &mut Vec<Value>, entry: Value) {
    let follows_kill = result.last().is_some_and(|previous| previous[0] == "pkill");
    if entry[0] == "pkill" && follows_kill {
        result.pop();
        result.push(json!(["close_browsers"]));
    } else {
        result.push(entry);
    }
}

pub(crate) fn collapsed(log: Vec<Value>) -> Vec<Value> {
    let mut result = Vec::new();
    for entry in log {
        push_collapsing(&mut result, entry);
    }
    result
}

pub(crate) fn run_hold(world: &World, reports: &Reports) {
    let config = config();
    let local = LocalSeat::new(&config, world, world, world);
    let edges = HoldEdges {
        clock: world,
        probe: world,
        reporter: reports,
        local: &local,
    };
    let _ = hold_seat(&mut &*world, &mut &*world, &edges, world.spec.longest);
}

pub(crate) fn check_hold(spec: Hold) {
    let (log, expected_reports) = (spec.log.clone(), spec.reports.clone());
    let (world, reports) = (World::new(spec), Reports::default());
    run_hold(&world, &reports);
    assert_eq!(collapsed(world.log.borrow().clone()), log);
    assert_eq!(reports.0.borrow().clone(), expected_reports);
}

#[test]
pub(crate) fn every_hold_scenario_behaves_as_the_python_did() {
    let specs: Vec<Hold> =
        serde_json::from_str(include_str!("fixtures/hold.json")).unwrap_or_default();
    assert!(specs.len() >= 8);
    for spec in specs {
        check_hold(spec);
    }
}
