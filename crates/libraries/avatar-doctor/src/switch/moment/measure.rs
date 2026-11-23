use super::boundary::Boundary;
use super::moments::Moments;
use super::sources::{agent_pairs, decision_pairs, delivery_pairs, heard_pairs};
use super::switch::Switch;
use crate::switch::failure::LatencyError;
use crate::switch::predicate::{decided_on, heard_as};
use crate::switch::record::{Logged, Record};
use crate::switch::search::latest;

pub(crate) fn measured(
    logged: &[Logged],
    start: usize,
    index: usize,
    tenure_end: f64,
) -> Result<Switch, LatencyError> {
    let anchor = &logged[index];
    let moments = moments_of(logged, start, index, tenure_end)?;
    Ok(Switch {
        switched_at: switched_moment(&moments).unwrap_or(anchor.at),
        servant: servant_of(&anchor.record),
        pane: anchor.record.text("pane").map(str::to_owned),
        sequence: anchor.record.sequence(),
        delivered: anchor.record.text("delivered").map(str::to_owned),
        moments,
    })
}

fn moments_of(
    logged: &[Logged],
    start: usize,
    index: usize,
    tenure_end: f64,
) -> Result<Moments, LatencyError> {
    let anchor = &logged[index];
    let sequence = anchor.record.sequence();
    let heard = latest(logged, start, index, heard_as(sequence));
    let decision = latest(logged, start, index + 1, decided_on(sequence));
    let mut moments = Moments::unrecorded();
    moments.record_all(heard_pairs(heard));
    moments.record_all(decision_pairs(decision)?);
    moments.record_all(delivery_pairs(anchor)?);
    moments.record_all(agent_pairs(logged, index, tenure_end));
    Ok(moments)
}

fn switched_moment(moments: &Moments) -> Option<f64> {
    moments
        .get(Boundary::DecisionDone)
        .filter(|at| at.abs() > 0.0)
}

fn servant_of(record: &Record) -> Option<String> {
    match record.text("servant") {
        Some(servant) if !servant.is_empty() => Some(servant.to_owned()),
        _ => record.text("choice").map(str::to_owned),
    }
}
