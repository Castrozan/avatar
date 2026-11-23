use std::os::unix::process::ExitStatusExt;
use std::process::Command;

use crate::commands::Programs;

const STOPPED_UNITS: [&str; 9] = [
    "avatar-conversation.target",
    "avatar-listener.service",
    "avatar-remote-seat.service",
    "avatar.target",
    "avatar-camera.service",
    "avatar-browser.service",
    "avatar-control-server.service",
    "avatar-renderer.service",
    "avatar-audio.service",
];
const COMMAND_NOT_FOUND: i32 = 127;
const SIGNAL_EXIT_BASE: i32 = 128;

fn inherited_status(command: &[String]) -> i32 {
    let Some((program, arguments)) = command.split_first() else {
        return COMMAND_NOT_FOUND;
    };
    match Command::new(program).args(arguments).status() {
        Ok(status) => status
            .code()
            .or_else(|| status.signal().map(|signal| SIGNAL_EXIT_BASE + signal))
            .unwrap_or(COMMAND_NOT_FOUND),
        Err(_) => COMMAND_NOT_FOUND,
    }
}

pub fn avatar_stop(programs: &Programs, stop_audio: impl FnOnce() -> i32) -> i32 {
    let mut command = programs.words(&["systemctl", "--user", "stop"]);
    command.extend(STOPPED_UNITS.map(str::to_owned));
    let stop_status = inherited_status(&command);
    match stop_audio() {
        0 => stop_status,
        audio_status => audio_status,
    }
}
