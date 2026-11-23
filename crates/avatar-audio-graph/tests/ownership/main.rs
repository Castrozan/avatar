mod names;
mod properties;

use avatar_audio_graph::ownership::{AudioCommand, AudioPhase, decide, observe};
use avatar_audio_graph::pulse::module_record::ModuleRecord;
use avatar_audio_graph::state::record::AudioState;
use names::{action_named, command_named, phase_named};
use serde::Deserialize;

const FIXTURE: &str = include_str!("fixtures/ownership.json");

#[derive(Deserialize)]
struct OwnershipFixture {
    decisions: Vec<DecisionRow>,
    observations: Vec<ObservationRow>,
}

#[derive(Deserialize)]
struct DecisionRow {
    phase: String,
    command: String,
    action: String,
}

#[derive(Deserialize)]
struct ObservationRow {
    state: AudioState,
    modules: Vec<ModuleRecord>,
    phase: String,
}

fn fixture() -> Result<OwnershipFixture, String> {
    serde_json::from_str(FIXTURE).map_err(|error| error.to_string())
}

fn check_decision(row: &DecisionRow) -> Result<(), String> {
    let phase = phase_named(&row.phase)?;
    let command = command_named(&row.command)?;
    assert_eq!(
        decide(phase, command),
        action_named(&row.action)?,
        "{} {}",
        row.phase,
        row.command
    );
    Ok(())
}

fn check_observation(row: &ObservationRow) -> Result<(), String> {
    assert_eq!(observe(&row.state, &row.modules), phase_named(&row.phase)?);
    Ok(())
}

#[test]
fn decide_matches_every_python_transition_cell() -> Result<(), String> {
    let fixture = fixture()?;
    assert_eq!(
        fixture.decisions.len(),
        AudioPhase::ALL.len() * AudioCommand::ALL.len()
    );
    fixture.decisions.iter().try_for_each(check_decision)
}

#[test]
fn observe_matches_every_python_world_combination() -> Result<(), String> {
    let fixture = fixture()?;
    assert_eq!(fixture.observations.len(), 24);
    fixture.observations.iter().try_for_each(check_observation)
}
