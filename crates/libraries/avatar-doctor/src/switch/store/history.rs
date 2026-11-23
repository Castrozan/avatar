use std::cmp::Ordering;

use crate::switch::failure::LatencyError;
use crate::switch::moment::Switch;

pub const KEPT_SWITCHES: usize = 1000;

pub trait SwitchHistory {
    fn remembered(&self) -> Result<Vec<Switch>, LatencyError>;
    fn keep(&self, switches: &[Switch]) -> Result<(), LatencyError>;
}

#[must_use]
pub fn merged(remembered: Vec<Switch>, fresh: Vec<Switch>) -> Vec<Switch> {
    let mut combined: Vec<Switch> = remembered.into_iter().chain(fresh).collect();
    combined.sort_by(by_moment);
    let unique = latest_per_moment(combined);
    let kept_from = unique.len().saturating_sub(KEPT_SWITCHES);
    unique.into_iter().skip(kept_from).collect()
}

fn by_moment(earlier: &Switch, later: &Switch) -> Ordering {
    earlier
        .switched_at
        .partial_cmp(&later.switched_at)
        .unwrap_or(Ordering::Equal)
}

fn latest_per_moment(ordered: Vec<Switch>) -> Vec<Switch> {
    let mut unique = Vec::with_capacity(ordered.len());
    for switch in ordered {
        push_latest(&mut unique, switch);
    }
    unique
}

fn push_latest(unique: &mut Vec<Switch>, switch: Switch) {
    if unique.last().is_some_and(|last| same_moment(last, &switch)) {
        unique.pop();
    }
    unique.push(switch);
}

fn same_moment(earlier: &Switch, later: &Switch) -> bool {
    earlier.switched_at.partial_cmp(&later.switched_at) == Some(Ordering::Equal)
}
