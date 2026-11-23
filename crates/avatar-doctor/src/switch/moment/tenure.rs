use super::measure::measured;
use super::switch::Switch;
use crate::switch::failure::LatencyError;
use crate::switch::predicate::{
    is_anchor, is_conversation_switch, is_delivery, is_switch_decision,
};
use crate::switch::record::{Logged, Record};

struct Tenure {
    start: usize,
    index: usize,
    next: Option<usize>,
}

pub fn switches_in(logged: &[Logged]) -> Result<Vec<Switch>, LatencyError> {
    let positions = switch_positions(logged)?;
    tenures(&positions)
        .filter(|tenure| is_conversation_switch(&logged[tenure.index].record))
        .map(|tenure| measured_tenure(logged, &tenure))
        .collect()
}

fn measured_tenure(logged: &[Logged], tenure: &Tenure) -> Result<Switch, LatencyError> {
    let end = tenure_end(logged, tenure.next);
    measured(logged, tenure.start, tenure.index, end)
}

fn tenures(positions: &[usize]) -> impl Iterator<Item = Tenure> + '_ {
    let starts = std::iter::once(0).chain(positions.iter().copied());
    starts
        .zip(positions.iter().copied())
        .zip(followers(positions))
        .map(|((start, index), next)| Tenure { start, index, next })
}

fn followers(positions: &[usize]) -> impl Iterator<Item = Option<usize>> + '_ {
    positions
        .iter()
        .skip(1)
        .copied()
        .map(Some)
        .chain(std::iter::once(None))
}

fn switch_positions(logged: &[Logged]) -> Result<Vec<usize>, LatencyError> {
    let anchors = anchor_positions(logged);
    anchors
        .iter()
        .copied()
        .zip(followers(&anchors))
        .filter_map(|(index, next)| switch_position(logged, index, next))
        .collect()
}

fn anchor_positions(logged: &[Logged]) -> Vec<usize> {
    (0..logged.len())
        .filter(|&index| is_anchor(&logged[index].record))
        .collect()
}

fn switch_position(
    logged: &[Logged],
    index: usize,
    next: Option<usize>,
) -> Option<Result<usize, LatencyError>> {
    match answered(logged, index, next) {
        Ok(true) => None,
        Ok(false) => Some(Ok(index)),
        Err(error) => Some(Err(error)),
    }
}

fn answered(logged: &[Logged], index: usize, next: Option<usize>) -> Result<bool, LatencyError> {
    let decision = &logged[index].record;
    let upcoming = next.map(|next_index| &logged[next_index].record);
    match upcoming {
        Some(upcoming) if is_switch_decision(decision) => answered_by(upcoming, decision),
        _ => Ok(false),
    }
}

fn answered_by(upcoming: &Record, decision: &Record) -> Result<bool, LatencyError> {
    if !is_delivery(upcoming) {
        return Ok(false);
    }
    Ok(upcoming.sequence() == decision.required_sequence()?)
}

fn tenure_end(logged: &[Logged], next: Option<usize>) -> f64 {
    next.map_or(f64::INFINITY, |next_index| logged[next_index].at)
}
