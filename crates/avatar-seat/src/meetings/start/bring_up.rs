use super::channels::audio_command;
use super::targets::Target;
use super::wait::{ReadinessWait, wait_until_ready};
use super::{
    AUDIO_COMMAND_TIMEOUT_SECONDS, AUDIO_REPAIR_TIMEOUT_SECONDS, READINESS_TIMEOUT_SECONDS,
    StartEdges,
};
use crate::local::SeatFailure;
use crate::meetings::readiness::Check;
use crate::meetings::run_checked;

pub struct BringUp {
    pub target: Target,
    pub checks: Vec<Check>,
}

impl BringUp {
    #[must_use]
    pub fn meeting() -> Self {
        Self {
            target: Target::Meeting,
            checks: vec![Check::Renderer, Check::Camera],
        }
    }

    fn waiting(&self) -> ReadinessWait {
        let (units, checks) = (self.target.units(), self.checks.clone());
        ReadinessWait {
            units,
            checks,
            timeout_seconds: READINESS_TIMEOUT_SECONDS,
        }
    }
}

fn repair_audio(edges: &StartEdges<'_>) -> Result<(), SeatFailure> {
    let command = audio_command(&["start"]);
    if !edges.readiness.audio_ready()? {
        run_checked(
            edges.processes,
            &command,
            AUDIO_REPAIR_TIMEOUT_SECONDS,
            true,
        )?;
    }
    Ok(())
}

fn require_audio(edges: &StartEdges<'_>) -> Result<(), SeatFailure> {
    if edges.readiness.audio_ready()? {
        return Ok(());
    }
    Err(SeatFailure::Runtime(
        "The avatar audio graph did not become ready".to_owned(),
    ))
}

fn start_target(edges: &StartEdges<'_>, plan: &BringUp) -> Result<(), SeatFailure> {
    repair_audio(edges)?;
    let start = plan.target.systemctl("start");
    run_checked(edges.processes, &start, AUDIO_REPAIR_TIMEOUT_SECONDS, false)?;
    wait_until_ready(edges, &plan.waiting())?;
    require_audio(edges)
}

fn stop_target(edges: &StartEdges<'_>, target: Target) -> Result<(), SeatFailure> {
    let command = target.systemctl("stop");
    edges
        .processes
        .run(&command, AUDIO_COMMAND_TIMEOUT_SECONDS, false)?;
    Ok(())
}

pub fn bring_up(edges: &StartEdges<'_>, plan: &BringUp) -> Result<(), SeatFailure> {
    let outcome = start_target(edges, plan);
    if outcome.is_err() {
        stop_target(edges, plan.target)?;
    }
    outcome
}
