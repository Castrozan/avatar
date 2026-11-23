use serde_json::Value;

use crate::edges::{EdgeFailure, Processes, SeatProbe};
use crate::support::truth::python_truthy;
use crate::support::words::joined_words;

const PROBE_TIMEOUT_SECONDS: u64 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Check {
    Renderer,
    Camera,
}

pub trait Readiness {
    fn audio_ready(&self) -> Result<bool, EdgeFailure>;
    fn renderer_ready(&self) -> Result<bool, EdgeFailure>;
    fn camera_ready(&self) -> Result<bool, EdgeFailure>;
}

pub fn check_ready(readiness: &dyn Readiness, check: Check) -> Result<bool, EdgeFailure> {
    match check {
        Check::Renderer => readiness.renderer_ready(),
        Check::Camera => readiness.camera_ready(),
    }
}

pub struct SystemReadiness<'edges> {
    pub processes: &'edges dyn Processes,
    pub control_health: &'edges dyn SeatProbe,
}

fn control_flag(health: Option<Value>, key: &str) -> bool {
    let found = health.and_then(|body| body.get(key).cloned());
    found.is_some_and(|value| python_truthy(&value))
}

fn running_of(output: &str) -> Result<bool, EdgeFailure> {
    let status: Value =
        serde_json::from_str(output).map_err(|error| EdgeFailure::Malformed(error.to_string()))?;
    Ok(status.get("running").is_some_and(python_truthy))
}

fn audio_running(processes: &dyn Processes) -> Result<bool, EdgeFailure> {
    let command = joined_words(&[&["avatar-audio", "status"]]);
    let result = processes.run(&command, PROBE_TIMEOUT_SECONDS, true)?;
    if result.code != 0 {
        return Ok(false);
    }
    running_of(&result.stdout)
}

impl SystemReadiness<'_> {
    #[must_use]
    pub fn control_server_ready(&self) -> bool {
        let health = self.control_health.health();
        let status = health.and_then(|body| body.get("status").cloned());
        status.is_some_and(|status| status == "ok")
    }
}

impl Readiness for SystemReadiness<'_> {
    fn audio_ready(&self) -> Result<bool, EdgeFailure> {
        audio_running(self.processes)
    }

    fn renderer_ready(&self) -> Result<bool, EdgeFailure> {
        Ok(control_flag(
            self.control_health.health(),
            "rendererConnected",
        ))
    }

    fn camera_ready(&self) -> Result<bool, EdgeFailure> {
        let command = joined_words(&[&["v4l2-ctl", "-d", "/dev/video10", "--info"]]);
        let capture = self.processes.run(&command, PROBE_TIMEOUT_SECONDS, true)?;
        Ok(capture.code == 0 && capture.stdout.contains("Video Capture"))
    }
}
