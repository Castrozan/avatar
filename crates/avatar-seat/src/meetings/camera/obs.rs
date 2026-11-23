use super::CameraStore;
use super::launch::launch_command;
use super::layout::{
    CAMERA_ACTIVE, CAMERA_NOT_SENDING, COLLECTION_FLAG, CameraConfig, OWNED_INSTANCE_END_SIGNAL,
};
use crate::edges::{Clock, Processes};
use crate::holder::{CameraFailure, MeetingCamera};
use crate::local::{COMMAND_TIMEOUT_SECONDS, SeatFailure, end_matching};
use crate::meetings::run_checked;
use crate::support::words::joined_words;

pub struct ObsCamera<'edges> {
    pub config: &'edges CameraConfig,
    pub processes: &'edges dyn Processes,
    pub clock: &'edges dyn Clock,
    pub store: &'edges dyn CameraStore,
}

fn runtime(message: String) -> SeatFailure {
    SeatFailure::Runtime(message)
}

fn require_installed(camera: &ObsCamera<'_>) -> Result<(), SeatFailure> {
    let application = &camera.config.application;
    if camera.store.exists(application) {
        return Ok(());
    }
    Err(runtime(format!(
        "OBS Studio is not installed: {application} is missing"
    )))
}

fn require_active_report(camera: &ObsCamera<'_>) -> Result<(), SeatFailure> {
    match camera.reported_camera_state().as_deref() {
        Some(CAMERA_ACTIVE) => Ok(()),
        _ => Err(runtime(CAMERA_NOT_SENDING.to_owned())),
    }
}

fn launch(camera: &ObsCamera<'_>) -> Result<(), SeatFailure> {
    let command = launch_command();
    run_checked(camera.processes, &command, COMMAND_TIMEOUT_SECONDS, false)?;
    Ok(())
}

fn start(camera: &ObsCamera<'_>) -> Result<(), SeatFailure> {
    require_installed(camera)?;
    camera.write_configuration()?;
    camera.stop()?;
    launch(camera)?;
    require_active_report(camera)
}

impl ObsCamera<'_> {
    pub fn stop(&self) -> Result<(), SeatFailure> {
        end_matching(self.processes, OWNED_INSTANCE_END_SIGNAL, COLLECTION_FLAG)?;
        Ok(self.store.remove(&self.config.report_path())?)
    }

    pub fn running(&self) -> Result<bool, SeatFailure> {
        let command = joined_words(&[&["/usr/bin/pgrep", "-f", "--", COLLECTION_FLAG]]);
        let found = self
            .processes
            .run(&command, COMMAND_TIMEOUT_SECONDS, true)?;
        Ok(found.code == 0)
    }

    pub fn start(&self) -> Result<(), SeatFailure> {
        start(self)
    }
}

fn reported<Value>(outcome: Result<Value, SeatFailure>) -> Result<Value, CameraFailure> {
    outcome.map_err(|failure| CameraFailure(failure.to_string()))
}

impl MeetingCamera for ObsCamera<'_> {
    fn started(&mut self) -> Result<bool, CameraFailure> {
        Ok(ObsCamera::started(self))
    }

    fn running(&mut self) -> Result<bool, CameraFailure> {
        reported(ObsCamera::running(self))
    }

    fn start(&mut self) -> Result<(), CameraFailure> {
        reported(ObsCamera::start(self))
    }

    fn stop(&mut self) -> Result<(), CameraFailure> {
        reported(ObsCamera::stop(self))
    }
}
