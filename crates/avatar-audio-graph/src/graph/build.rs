use std::collections::BTreeSet;

use super::teardown::tear_down;
use super::{AudioGraph, apply};
use crate::effects::names::reserved_node_names;
use crate::effects::plan::mix_plan;
use crate::errors::AudioGraphError;
use crate::pulse::control::PulseControl;
use crate::pulse::listing::node_names;
use crate::state::record::AudioState;
use crate::state::store::StateStore;

pub(super) fn build<P: PulseControl, S: StateStore>(
    graph: &mut AudioGraph<P, S>,
) -> Result<(), AudioGraphError> {
    ensure_names_available(graph)?;
    graph.state = AudioState::default();
    let outcome = create_ready_mix(graph);
    if let Err(error) = outcome {
        tear_down(graph)?;
        return Err(error);
    }
    Ok(())
}

pub(super) fn rebuild<P: PulseControl, S: StateStore>(
    graph: &mut AudioGraph<P, S>,
) -> Result<(), AudioGraphError> {
    tear_down(graph)?;
    build(graph)
}

fn create_ready_mix<P: PulseControl, S: StateStore>(
    graph: &mut AudioGraph<P, S>,
) -> Result<(), AudioGraphError> {
    for step in mix_plan() {
        apply(graph, step)?;
    }
    graph.state.ready = true;
    graph.store.save(&graph.state)?;
    Ok(())
}

fn ensure_names_available<P: PulseControl, S: StateStore>(
    graph: &mut AudioGraph<P, S>,
) -> Result<(), AudioGraphError> {
    let existing = existing_node_names(graph)?;
    let conflicts: Vec<String> = reserved_node_names()
        .into_iter()
        .filter(|name| existing.contains(*name))
        .map(str::to_owned)
        .collect();
    if !conflicts.is_empty() {
        return Err(AudioGraphError::NameCollision(conflicts));
    }
    Ok(())
}

fn existing_node_names<P: PulseControl, S: StateStore>(
    graph: &mut AudioGraph<P, S>,
) -> Result<BTreeSet<String>, AudioGraphError> {
    let mut existing = BTreeSet::new();
    existing.extend(list_node_names(graph, &["--format=json", "list", "sinks"])?);
    existing.extend(list_node_names(
        graph,
        &["--format=json", "list", "sources"],
    )?);
    Ok(existing)
}

pub(super) fn list_node_names<P: PulseControl, S: StateStore>(
    graph: &mut AudioGraph<P, S>,
    arguments: &[&str],
) -> Result<Vec<String>, AudioGraphError> {
    let output = graph.pulse.run(arguments)?;
    Ok(node_names(&output)?)
}
