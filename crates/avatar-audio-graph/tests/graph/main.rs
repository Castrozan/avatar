mod fixture;
mod levels;
mod probe;
mod replay;
mod requests;

use std::cell::RefCell;
use std::rc::Rc;

use avatar_audio_graph::graph::AudioGraph;
use avatar_audio_graph::state::record::AudioState;
use fixture::{ScenarioFixture, ScenarioRecord, StepRecord};
use probe::ProbeStore;
use replay::{
    Probe, ReplayScript, Script, ScriptedPulse, load_script, perform_step, script_verdict,
};

const FIXTURE: &str = include_str!("fixtures/scenarios.json");

type Graph = AudioGraph<ScriptedPulse, ProbeStore>;
type Outcome = Result<(), String>;

struct Harness {
    graph: Graph,
    script: Script,
    probe: Probe,
    outputs: Vec<String>,
}

impl Harness {
    fn new(outputs: &[String]) -> Self {
        let script: Script = Rc::new(RefCell::new(ReplayScript::default()));
        let probe: Probe = Rc::new(RefCell::new(None));
        Self {
            graph: new_graph(&script, &probe),
            script,
            probe,
            outputs: outputs.to_vec(),
        }
    }
}

fn new_graph(script: &Script, probe: &Probe) -> Graph {
    AudioGraph::new(
        ScriptedPulse::new(Rc::clone(script)),
        ProbeStore::new(Rc::clone(probe)),
        AudioState::default(),
    )
}

#[test]
fn every_python_scenario_replays_with_identical_calls_state_and_errors() -> Result<(), String> {
    let fixture: ScenarioFixture =
        serde_json::from_str(FIXTURE).map_err(|error| error.to_string())?;
    assert_eq!(fixture.scenarios.len(), 23);
    fixture
        .scenarios
        .iter()
        .try_for_each(|scenario| replay_scenario(&fixture.outputs, scenario))
}

fn replay_scenario(outputs: &[String], scenario: &ScenarioRecord) -> Result<(), String> {
    let mut harness = Harness::new(outputs);
    scenario
        .steps
        .iter()
        .try_for_each(|record| replay_step(&mut harness, record))
        .map_err(|error| format!("{}: {error}", scenario.name))
}

fn replay_step(harness: &mut Harness, record: &StepRecord) -> Result<(), String> {
    let outcome = run_recorded(harness, record)?;
    check_outcome(&outcome, record)?;
    check_state(harness, record)
}

fn run_recorded(harness: &mut Harness, record: &StepRecord) -> Result<Outcome, String> {
    load_script(&harness.script, &record.calls, &harness.outputs)?;
    let outcome = perform_step(&mut harness.graph, &record.step);
    script_verdict(&harness.script)?;
    Ok(outcome)
}

fn check_outcome(outcome: &Outcome, record: &StepRecord) -> Result<(), String> {
    let expected_ok = record.result.ok == Some(true);
    match (outcome, &record.result.error) {
        (Ok(()), None) if expected_ok => Ok(()),
        (Err(actual), Some(expected)) if actual == expected => Ok(()),
        _ => Err(format!(
            "{} outcome {outcome:?}, expected {:?}",
            record.step.action, record.result
        )),
    }
}

fn check_state(harness: &Harness, record: &StepRecord) -> Result<(), String> {
    if harness.graph.state() != &record.state {
        return Err(format!("{} state {:?}", record.step.action, record.state));
    }
    if harness.probe.borrow().is_some() != record.file_exists {
        return Err(format!("{} file existence mismatch", record.step.action));
    }
    Ok(())
}
