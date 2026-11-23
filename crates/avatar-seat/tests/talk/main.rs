mod call;
mod fakes;
mod proptests;

use std::collections::HashMap;

use avatar_seat::edges::EdgeFailure;
use avatar_seat::local::{LocalSeat, SeatConfig};
use avatar_seat::talk::{RemoteTalk, TalkReport};
use call::Run;
use fakes::Machine;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
struct Scenario {
    arguments: Vec<String>,
    runs: Vec<Run>,
    healths: Vec<Value>,
    env: HashMap<String, String>,
    spawn_fails: bool,
    exit: Option<i32>,
    crash: Option<String>,
    stdout: String,
    stderr: String,
    calls: Vec<Value>,
}

fn config() -> SeatConfig {
    SeatConfig {
        avatar_host: "chise".to_owned(),
        browser_application: "/nix/store/browser/Google Chrome for Testing.app".to_owned(),
        home: "<home>".to_owned(),
        python: "<python>".to_owned(),
        holder_path: "<holder>".to_owned(),
    }
}

fn play(scenario: &Scenario) -> (Machine, Result<TalkReport, EdgeFailure>) {
    let machine = Machine::new(
        scenario.runs.clone(),
        scenario.healths.clone(),
        scenario.spawn_fails,
    );
    let config = config();
    let local = LocalSeat::new(&config, &machine, &machine, &machine);
    let outcome = RemoteTalk::new(local, &scenario.env).run(&scenario.arguments);
    (machine, outcome)
}

fn expected_report(scenario: &Scenario) -> (Option<i32>, String, String) {
    (
        scenario.exit,
        scenario.stdout.clone(),
        scenario.stderr.clone(),
    )
}

fn observed_report(report: TalkReport) -> (Option<i32>, String, String) {
    (Some(report.code), report.stdout, report.stderr)
}

fn check_outcome(scenario: &Scenario, outcome: Result<TalkReport, EdgeFailure>) {
    match outcome {
        Ok(report) => assert_eq!(observed_report(report), expected_report(scenario)),
        Err(_) => assert_eq!(scenario.crash.as_deref(), Some("OSError")),
    }
}

fn check_scenario(scenario: &Scenario) {
    let (machine, outcome) = play(scenario);
    assert_eq!(machine.calls(), scenario.calls, "{:?}", scenario.arguments);
    check_outcome(scenario, outcome);
}

#[test]
fn every_talk_scenario_behaves_as_the_python_did() {
    let scenarios: Vec<Scenario> =
        serde_json::from_str(include_str!("fixtures/talk.json")).unwrap_or_default();
    assert!(scenarios.len() >= 50);
    scenarios.iter().for_each(check_scenario);
}
