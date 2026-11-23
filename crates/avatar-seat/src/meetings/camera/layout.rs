use crate::local::RENDERER_PORT;

pub(crate) const OBS_SUPPORT: &str = "Library/Application Support/obs-studio/basic";
pub(crate) const PROFILE_NAME: &str = "Avatar";
pub(crate) const COLLECTION_NAME: &str = "Avatar";
pub(crate) const SCENE_NAME: &str = "Avatar";
pub(crate) const SOURCE_NAME: &str = "Avatar View";
pub(crate) const FRAME_WIDTH: u32 = 1280;
pub(crate) const FRAME_HEIGHT: u32 = 720;
pub(crate) const FRAMES_PER_SECOND: u32 = 30;
pub(crate) const CAMERA_REPORT: &str = ".local/share/avatar/virtual-camera-report";
pub(crate) const CAMERA_ACTIVE: &str = "active";
pub(crate) const CAMERA_REPORT_WAIT_SECONDS: f64 = 30.0;
pub(crate) const OWNED_INSTANCE_END_SIGNAL: &str = "KILL";
pub(crate) const COLLECTION_FLAG: &str = "--collection Avatar";
pub(crate) const CAMERA_NOT_SENDING: &str = "OBS did not start its virtual camera. Allow it once in System Settings, General, Login Items & Extensions, Camera Extensions, then run avatar-start again";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CameraConfig {
    pub home: String,
    pub application: String,
    pub start_script: String,
}

pub(crate) fn avatar_view_url() -> String {
    format!("http://localhost:{RENDERER_PORT}/?stage&bare")
}

impl CameraConfig {
    pub(crate) fn report_path(&self) -> String {
        format!("{}/{CAMERA_REPORT}", self.home)
    }

    pub(crate) fn report_directory(&self) -> String {
        format!("{}/.local/share/avatar", self.home)
    }

    pub(crate) fn support_path(&self) -> String {
        format!("{}/{OBS_SUPPORT}", self.home)
    }

    pub(crate) fn scenes_directory(&self) -> String {
        format!("{}/scenes", self.support_path())
    }

    pub(crate) fn profile_directory(&self) -> String {
        format!("{}/profiles/{PROFILE_NAME}", self.support_path())
    }
}
