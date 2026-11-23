use avatar_seat::edges::{SystemClock, SystemProcesses};
use avatar_seat::meetings::camera::{CameraConfig, FileSystemStore, ObsCamera};

use super::MacRoom;
use crate::failure::Failure;
use crate::meeting::MeetingCamera;

pub struct ObsMeetingCamera {
    pub config: CameraConfig,
}

fn failed(error: &impl ToString) -> Failure {
    Failure::runtime(&error.to_string())
}

impl ObsMeetingCamera {
    fn with_camera<Output>(&self, body: impl FnOnce(&ObsCamera<'_>) -> Output) -> Output {
        let clock = SystemClock::new();
        body(&ObsCamera {
            config: &self.config,
            processes: &SystemProcesses,
            clock: &clock,
            store: &FileSystemStore,
        })
    }
}

impl<Talk> MeetingCamera for MacRoom<Talk> {
    fn start_camera(&mut self) -> Result<(), Failure> {
        self.camera
            .with_camera(|camera| camera.start().map_err(|error| failed(&error)))
    }

    fn stop_camera(&mut self) -> Result<(), Failure> {
        self.camera
            .with_camera(|camera| camera.stop().map_err(|error| failed(&error)))
    }

    fn camera_running(&mut self) -> bool {
        self.camera
            .with_camera(|camera| camera.running().unwrap_or(false))
    }
}
