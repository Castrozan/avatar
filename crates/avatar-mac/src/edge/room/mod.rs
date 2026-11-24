mod console;
mod devices;
mod forwarding;
mod meeting;
mod probe;
mod reports;
mod seat;
mod session;
mod talk;

use std::path::PathBuf;

use avatar_listening::listener::session::SessionDirectory;
use avatar_listening::listener::units::LaunchdUnits;
use avatar_listening::user_units::launchd::process::ProcessLaunchctl;
use avatar_seat::local::SeatConfig;
use avatar_seat::meetings::camera::CameraConfig;

pub use console::StandardConsole;
pub use devices::{ObsMeetingCamera, SystemCallDefaults};
pub use seat::MacSeatControl;
pub use talk::{MacHostTalk, MacLiveOpening};

use super::client::SeatClient;
use super::stage::{BrowserProcesses, LaunchdStageUnits, MacSurface, SystemWaiting};
use crate::failure::Failure;

const OPEN_PROGRAM: &str = "/usr/bin/open";

pub struct MacStage {
    pub units: LaunchdStageUnits,
    pub surface: MacSurface,
    pub waiting: SystemWaiting,
}

pub struct MacSession {
    pub directory: SessionDirectory,
    pub units: LaunchdUnits,
}

pub struct MacRoom<Talk = ()> {
    pub stage: MacStage,
    pub session: MacSession,
    pub seat: MacSeatControl,
    pub camera: ObsMeetingCamera,
    pub call_defaults: SystemCallDefaults,
    pub config: SeatConfig,
    pub talk: Talk,
}

pub struct MacRoomSettings {
    pub config: SeatConfig,
    pub session_directory: PathBuf,
    pub user_identifier: u32,
    pub camera: CameraConfig,
    pub avatar_command: PathBuf,
    pub launchctl: PathBuf,
    pub audio_switcher: PathBuf,
}

fn launchctl_of(settings: &MacRoomSettings) -> ProcessLaunchctl {
    ProcessLaunchctl::new(settings.launchctl.clone())
}

fn stage_of(settings: &MacRoomSettings) -> MacStage {
    MacStage {
        units: LaunchdStageUnits::new(launchctl_of(settings), settings.user_identifier),
        surface: MacSurface {
            avatar_command: settings.avatar_command.clone(),
            open: PathBuf::from(OPEN_PROGRAM),
            config: settings.config.clone(),
            processes: BrowserProcesses::system(),
        },
        waiting: SystemWaiting::new(),
    }
}

fn session_of(settings: &MacRoomSettings) -> MacSession {
    MacSession {
        directory: SessionDirectory::new(settings.session_directory.clone()),
        units: LaunchdUnits::new(launchctl_of(settings), settings.user_identifier),
    }
}

fn seat_of(home: &str) -> Result<MacSeatControl, Failure> {
    Ok(MacSeatControl {
        client: SeatClient::on_this_mac()?,
        processes: BrowserProcesses::system(),
        profile_flag: MacSeatControl::profile_flag_for(home),
    })
}

impl MacRoom {
    pub fn on_this_mac(settings: MacRoomSettings) -> Result<Self, Failure> {
        Ok(Self {
            stage: stage_of(&settings),
            session: session_of(&settings),
            seat: seat_of(&settings.config.home)?,
            call_defaults: devices::call_defaults_of(&settings),
            camera: devices::camera_of(settings.camera),
            config: settings.config,
            talk: (),
        })
    }
}
