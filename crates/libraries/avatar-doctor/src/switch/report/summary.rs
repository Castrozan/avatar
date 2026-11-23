use std::cmp::Ordering;

use super::format::seconds;
use super::stage::{END_TO_END, STAGES, Stage, took};
use crate::numeric::count_as_float;
use crate::switch::moment::Switch;

const PERCENTILES: [u32; 2] = [50, 95];
const STAGE_WIDTH: usize = 25;
const MEASURED_WIDTH: usize = 10;

#[must_use]
pub fn summary_lines(switches: &[Switch]) -> Vec<String> {
    let mut lines = vec![title(switches.len()), header_line()];
    lines.extend(all_stages().map(|stage| stage_row(stage, switches)));
    lines
}

fn title(count: usize) -> String {
    format!("Across {count} switches (nearest-rank percentiles)")
}

fn all_stages() -> impl Iterator<Item = Stage> {
    STAGES.into_iter().chain([END_TO_END])
}

fn header_line() -> String {
    let percentiles = PERCENTILES
        .iter()
        .map(|percentile| format!("p{percentile:<11}"))
        .collect::<Vec<String>>()
        .concat();
    let titles = format!("  {:<STAGE_WIDTH$}{:<MEASURED_WIDTH$}", "stage", "measured");
    format!("{titles}{percentiles}").trim_end().to_owned()
}

fn stage_row(stage: Stage, switches: &[Switch]) -> String {
    let durations = durations_of(stage, switches);
    let measured = format!(
        "  {:<STAGE_WIDTH$}{:<MEASURED_WIDTH$}",
        stage.name,
        durations.len()
    );
    format!("{measured}{}", percentile_cells(&durations))
        .trim_end()
        .to_owned()
}

fn percentile_cells(durations: &[f64]) -> String {
    PERCENTILES
        .iter()
        .map(|percentile| format!("{:<12}", seconds(nearest_rank(durations, *percentile))))
        .collect::<Vec<String>>()
        .concat()
}

fn durations_of(stage: Stage, switches: &[Switch]) -> Vec<f64> {
    switches
        .iter()
        .filter_map(|switch| took(&switch.moments, stage.start, stage.end))
        .collect()
}

#[must_use]
pub fn nearest_rank(values: &[f64], percentile: u32) -> Option<f64> {
    let ordered = sorted(values);
    let target = f64::from(percentile) / 100.0 * count_as_float(ordered.len());
    let rank = (1..=ordered.len()).find(|rank| count_as_float(*rank) >= target)?;
    ordered.get(rank - 1).copied()
}

fn sorted(values: &[f64]) -> Vec<f64> {
    let mut ordered = values.to_vec();
    ordered.sort_by(|earlier, later| earlier.partial_cmp(later).unwrap_or(Ordering::Equal));
    ordered
}
