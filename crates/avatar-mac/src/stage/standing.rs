use super::ports::Stage;
use super::{
    CONTROL_PORT, NOT_READY, POLL_SECONDS, READY_WAIT_SECONDS, RENDERER_UNIT, STAGE_UNITS,
};
use crate::failure::Failure;

pub fn standing_already(stage: &mut impl Stage) -> bool {
    STAGE_UNITS.iter().all(|unit| stage.unit_active(unit))
}

fn start_unless_active(stage: &mut impl Stage, unit: &str) -> Result<(), Failure> {
    if stage.unit_active(unit) {
        return Ok(());
    }
    stage.start_unit(unit)
}

fn stop_if_active(stage: &mut impl Stage, unit: &str) -> Result<(), Failure> {
    if stage.unit_active(unit) {
        return stage.stop_unit(unit);
    }
    Ok(())
}

fn pause_before_retry(stage: &mut impl Stage, deadline: f64, failure: &str) -> Result<(), Failure> {
    if stage.monotonic() > deadline {
        return Err(Failure::timeout(failure));
    }
    stage.sleep(POLL_SECONDS);
    Ok(())
}

pub fn start_and_wait(
    stage: &mut impl Stage,
    units: &[&str],
    port: u16,
    failure: &str,
) -> Result<(), Failure> {
    units
        .iter()
        .try_for_each(|unit| start_unless_active(stage, unit))?;
    let deadline = stage.monotonic() + READY_WAIT_SECONDS;
    while !stage.answers(port) {
        pause_before_retry(stage, deadline, failure)?;
    }
    Ok(())
}

pub fn set_up(stage: &mut impl Stage) -> Result<(), Failure> {
    start_and_wait(stage, &STAGE_UNITS, CONTROL_PORT, NOT_READY)
}

pub fn strike(stage: &mut impl Stage) -> Result<(), Failure> {
    stage.hide_window_surface()?;
    let units = STAGE_UNITS.iter().chain([&RENDERER_UNIT]);
    units
        .into_iter()
        .try_for_each(|unit| stop_if_active(stage, unit))
}

pub fn with_standing<Output, Stand: Stage>(
    stage: &mut Stand,
    body: impl FnOnce(&mut Stand) -> Result<Output, Failure>,
) -> Result<Output, Failure> {
    let stood = standing_already(stage);
    set_up(stage)?;
    let outcome = body(stage);
    if !stood {
        strike(stage)?;
    }
    outcome
}
