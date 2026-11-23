use std::path::Path;

use super::CONTROL_PORT;
use super::RENDERER_PORT;
use super::SEAT_PORT;
use super::WINDOW_PORT;

const WAIT_WHILE_SEAT_SERVES: &str = "systemctl --user is-active --quiet avatar-remote-seat.service && systemctl --user start --wait avatar-remote-seat.service";
const SEAT_PROFILE: &str = ".local/share/avatar/seat-browser";
const WINDOW_PROFILE: &str = ".local/share/avatar/window-browser";
const SSH_OPTIONS: [&str; 9] = [
    "ssh",
    "-o",
    "BatchMode=yes",
    "-o",
    "ExitOnForwardFailure=yes",
    "-o",
    "ServerAliveInterval=5",
    "-o",
    "ServerAliveCountMax=3",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeatConfig {
    pub avatar_host: String,
    pub browser_application: String,
    pub home: String,
    pub python: String,
    pub holder_path: String,
}

impl SeatConfig {
    #[must_use]
    pub fn browser_name(&self) -> String {
        let stem = Path::new(&self.browser_application).file_stem();
        stem.map_or_else(String::new, |name| name.to_string_lossy().into_owned())
    }

    #[must_use]
    pub fn seat_profile_flag(&self) -> String {
        format!("--user-data-dir={}/{SEAT_PROFILE}", self.home)
    }

    #[must_use]
    pub fn window_profile_flag(&self) -> String {
        format!("--user-data-dir={}/{WINDOW_PROFILE}", self.home)
    }
}

pub(crate) fn tunnel_forward() -> String {
    format!("127.0.0.1:{SEAT_PORT}:127.0.0.1:{SEAT_PORT}")
}

fn window_forward() -> String {
    format!("127.0.0.1:{WINDOW_PORT}:127.0.0.1:{RENDERER_PORT}")
}

fn control_forward() -> String {
    format!("127.0.0.1:{CONTROL_PORT}:127.0.0.1:{CONTROL_PORT}")
}

fn owned(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_owned()).collect()
}

#[must_use]
pub fn tunnel_command(config: &SeatConfig) -> Vec<String> {
    let forwards = [tunnel_forward(), window_forward(), control_forward()];
    let tail = [config.avatar_host.as_str(), WAIT_WHILE_SEAT_SERVES];
    let mut command = owned(&SSH_OPTIONS);
    command.extend(
        forwards
            .into_iter()
            .flat_map(|forward| ["-L".to_owned(), forward]),
    );
    command.extend(owned(&tail));
    command
}
