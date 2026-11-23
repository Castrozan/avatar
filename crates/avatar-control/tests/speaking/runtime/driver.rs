use std::error::Error;
use std::path::Path;
use std::time::Duration;

use avatar_control::server::SpeechPort;

use avatar_control::speaking::runtime::SpeakingPort;
use serde_json::{Value, json};

use super::harness::{World, launch};
use super::scenario::steps::apply;

const GATE_SETTLE: Duration = Duration::from_millis(600);

const YIELD_ROUNDS: usize = 200;

pub(super) struct Outcome {
    pub(super) trace: Vec<Value>,
    pub(super) gate_commands: Vec<String>,
}

async fn settle(world: &World) {
    if world.script.gate_status.is_some() {
        return tokio::time::sleep(GATE_SETTLE).await;
    }
    for _ in 0..YIELD_ROUNDS {
        tokio::task::yield_now().await;
    }
}

fn record_phase(world: &World) {
    world.shared.with(|recording| {
        recording.drain();
        let phase = recording.phase.clone();
        recording.trace.push(json!({"phase": phase}));
    });
}

fn gate_commands(directory: &Path) -> Vec<String> {
    let log = std::fs::read_to_string(directory.join("calls.log")).unwrap_or_default();
    log.lines()
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .collect()
}

async fn play_steps(scenario: &Value, port: &mut SpeakingPort, world: &World) {
    for step in scenario["steps"].as_array().into_iter().flatten() {
        apply(step, port, world);
        settle(world).await;
        record_phase(world);
    }
}

fn attach_events(port: &mut SpeakingPort, world: &World) -> Result<(), Box<dyn Error>> {
    let events = port.take_events().ok_or("events already taken")?;
    world.shared.with(|recording| recording.attach(events));
    Ok(())
}

pub(super) async fn run_scenario(
    scenario: &Value,
    directory: &Path,
) -> Result<Outcome, Box<dyn Error>> {
    let (mut port, world) = launch(scenario, directory)?;
    attach_events(&mut port, &world)?;
    play_steps(scenario, &mut port, &world).await;
    let trace = world.shared.with(|recording| recording.trace.clone());
    Ok(Outcome {
        trace,
        gate_commands: gate_commands(directory),
    })
}
