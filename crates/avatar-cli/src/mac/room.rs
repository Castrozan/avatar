use std::io;
use std::path::PathBuf;
use std::process::ExitCode;

use avatar_mac::edge::room::{MacRoom, MacRoomSettings};
use avatar_seat::local::SeatConfig;
use avatar_seat::meetings::camera::CameraConfig;
use nix::unistd::getuid;

use super::environment::seat_config;
use crate::support::runtime::session_directory;

pub(super) const LAUNCHCTL: &str = "/bin/launchctl";
const OBS_APPLICATION: &str = "/Applications/OBS.app";
const CAMERA_START_SCRIPT: &str = ".local/share/avatar/start-virtual-camera.lua";

pub(super) fn camera_of(config: &SeatConfig) -> CameraConfig {
    CameraConfig {
        home: config.home.clone(),
        application: OBS_APPLICATION.to_owned(),
        start_script: format!("{}/{CAMERA_START_SCRIPT}", config.home),
    }
}

fn settings_of(config: SeatConfig) -> MacRoomSettings {
    MacRoomSettings {
        camera: camera_of(&config),
        config,
        session_directory: session_directory(),
        user_identifier: getuid().as_raw(),
        avatar_command: PathBuf::from("avatar-command"),
        launchctl: PathBuf::from(LAUNCHCTL),
    }
}

pub(super) fn room() -> io::Result<MacRoom> {
    let settings = settings_of(seat_config()?);
    MacRoom::on_this_mac(settings).map_err(|failure| io::Error::other(failure.message))
}

pub(super) fn code_of(code: i32) -> ExitCode {
    ExitCode::from(u8::try_from(code).unwrap_or(1))
}
