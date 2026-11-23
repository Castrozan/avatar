use super::clock::{clock_text, wall_clock};
use super::format::{seconds, shown};
use super::span::recorded_span;
use super::stage::{END_TO_END, STAGES, Stage, took};
use crate::switch::failure::LatencyError;
use crate::switch::moment::{Boundary, Moments, Switch};
use jiff::tz::TimeZone;

const BOUNDARY_WIDTH: usize = 23;
const TIME_WIDTH: usize = 14;
const STAGE_WIDTH: usize = 25;

pub fn switch_lines(switch: &Switch, zone: &TimeZone) -> Result<Vec<String>, LatencyError> {
    let mut lines = vec![switch_header(switch, zone)?, column_titles()];
    lines.push(first_boundary_row(&switch.moments, zone)?);
    lines.extend(stage_rows(&switch.moments, zone)?);
    lines.push(end_to_end_row(&switch.moments));
    lines.push(format!(
        "  recorded span {}",
        recorded_span(&switch.moments)
    ));
    Ok(lines)
}

fn end_to_end_row(moments: &Moments) -> String {
    let total = took(moments, END_TO_END.start, END_TO_END.end);
    format!("  end to end {}", seconds(total))
}

fn switch_header(switch: &Switch, zone: &TimeZone) -> Result<String, LatencyError> {
    let switched = wall_clock(switch.switched_at, zone)?
        .strftime("%Y-%m-%d %H:%M:%S")
        .to_string();
    Ok(format!(
        "Switch to {} at {switched}, sequence {}, {}",
        shown(switch.servant.as_ref()),
        shown(switch.sequence.as_ref()),
        delivery_of(switch)
    ))
}

fn column_titles() -> String {
    format!(
        "  {:<BOUNDARY_WIDTH$}{:<TIME_WIDTH$}{:<STAGE_WIDTH$}took",
        "boundary", "time", "stage"
    )
}

fn first_boundary_row(moments: &Moments, zone: &TimeZone) -> Result<String, LatencyError> {
    let clock = clock_text(moments.get(Boundary::SpeechEnded), zone)?;
    Ok(format!(
        "  {:<BOUNDARY_WIDTH$}{clock}",
        Boundary::SpeechEnded.label()
    ))
}

fn stage_rows(moments: &Moments, zone: &TimeZone) -> Result<Vec<String>, LatencyError> {
    STAGES
        .iter()
        .map(|stage| stage_row(*stage, moments, zone))
        .collect()
}

fn stage_row(stage: Stage, moments: &Moments, zone: &TimeZone) -> Result<String, LatencyError> {
    let clock = clock_text(moments.get(stage.end), zone)?;
    Ok(format!(
        "  {:<BOUNDARY_WIDTH$}{clock:<TIME_WIDTH$}{:<STAGE_WIDTH$}{}",
        stage.end.label(),
        stage.name,
        seconds(took(moments, stage.start, stage.end))
    ))
}

fn delivery_of(switch: &Switch) -> String {
    match &switch.delivered {
        None => "no prompt delivery recorded".to_owned(),
        Some(delivered) => format!(
            "delivered as {delivered} to {}",
            shown(switch.pane.as_ref())
        ),
    }
}
