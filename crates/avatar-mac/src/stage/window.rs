use super::commands::present;
use super::ports::Stage;
use super::standing::start_and_wait;
use super::{
    READY_WAIT_SECONDS, RENDERER_NOT_READY, RENDERER_PAGE, RENDERER_PORT, RENDERER_UNIT,
    WINDOW_POLL_SECONDS,
};
use crate::failure::{Failure, FailureKind};

fn back_off(stage: &mut impl Stage, refusal: Failure, deadline: f64) -> Result<(), Failure> {
    if stage.monotonic() > deadline {
        return Err(refusal);
    }
    stage.sleep(WINDOW_POLL_SECONDS);
    Ok(())
}

fn present_until(stage: &mut impl Stage, deadline: f64) -> Result<(), Failure> {
    match present(stage, true) {
        Err(refusal) if refusal.kind == FailureKind::CalledProcess => {
            back_off(stage, refusal, deadline)?;
            present_until(stage, deadline)
        }
        outcome => outcome,
    }
}

pub fn present_once_the_window_renders(stage: &mut impl Stage) -> Result<(), Failure> {
    let deadline = stage.monotonic() + READY_WAIT_SECONDS;
    present_until(stage, deadline)
}

pub fn show_window(stage: &mut impl Stage) -> Result<(), Failure> {
    start_and_wait(stage, &[RENDERER_UNIT], RENDERER_PORT, RENDERER_NOT_READY)?;
    stage.hide_window_surface()?;
    stage.open_window(RENDERER_PAGE, RENDERER_PORT)?;
    present_once_the_window_renders(stage)
}

pub fn hide_window(stage: &mut impl Stage) -> Result<(), Failure> {
    stage.hide_window_surface()
}
