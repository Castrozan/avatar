mod call_defaults;
mod camera;
mod direction;

use std::path::Path;

use avatar_seat::meetings::camera::CameraConfig;

pub use call_defaults::SystemCallDefaults;
pub use camera::ObsMeetingCamera;

use super::{MacRoom, MacRoomSettings};
use crate::failure::Failure;
use crate::meeting::CallDefaults;

pub(super) fn call_defaults_of(settings: &MacRoomSettings) -> SystemCallDefaults {
    let session: &Path = &settings.session_directory;
    SystemCallDefaults {
        switcher: settings.audio_switcher.clone(),
        record: session.join(call_defaults::CALL_DEFAULTS_RECORD),
    }
}

pub(super) const fn camera_of(config: CameraConfig) -> ObsMeetingCamera {
    ObsMeetingCamera { config }
}

impl<Talk> CallDefaults for MacRoom<Talk> {
    fn route_defaults_to_the_call(&mut self) -> Result<(), Failure> {
        self.call_defaults.route_to_the_call()
    }

    fn restore_defaults(&mut self) -> Result<(), Failure> {
        self.call_defaults.restore()
    }
}
