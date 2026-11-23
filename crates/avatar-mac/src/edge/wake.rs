use std::path::PathBuf;
use std::time::Duration;

use avatar_adapters::process::{Captured, run_captured};
use avatar_adapters::shell_quote::join;
use avatar_listening::listener::runtime::listening;
use avatar_listening::listener::units::LaunchdUnits;
use avatar_listening::user_units::launchd::process::ProcessLaunchctl;
use avatar_seat::edges::{HttpProbe, SeatProbe};
use avatar_wake::listener::{Boundary, BoundaryFailure};
use avatar_wake::mac::MacEdges;

use super::stage::SystemWaiting;
use crate::waiting::Clock;

const OPEN_TIMEOUT: Duration = Duration::from_mins(2);
const CONNECT_OPTIONS: [&str; 4] = ["-o", "BatchMode=yes", "-o", "ConnectTimeout=5"];

#[derive(Debug, Clone)]
pub struct MacWakeEdges {
    pub avatar_host: String,
    pub room: Option<String>,
    pub seat_port: u16,
    pub ssh: PathBuf,
    pub launchctl: PathBuf,
    pub user_identifier: u32,
    pub open_command: Vec<String>,
    pub clock: SystemWaiting,
}

fn unavailable(message: &impl ToString) -> BoundaryFailure {
    BoundaryFailure::Unavailable(message.to_string())
}

fn stdout_of(captured: Captured) -> Boundary<String> {
    if !captured.succeeded() {
        return Err(unavailable(&format!("exit status {}", captured.exit_code)));
    }
    String::from_utf8(captured.stdout).map_err(|error| BoundaryFailure::Invalid(error.to_string()))
}

impl MacWakeEdges {
    fn seat_flag(&self, key: &str) -> bool {
        let health = HttpProbe {
            port: self.seat_port,
        }
        .health();
        health.and_then(|body| body.get(key).and_then(serde_json::Value::as_bool)) == Some(true)
    }

    fn ssh_command(&self, command: &[String]) -> Vec<String> {
        let program = self.ssh.to_string_lossy().into_owned();
        let options = CONNECT_OPTIONS.map(str::to_owned);
        let remote = [self.avatar_host.clone(), join(command)];
        std::iter::once(program)
            .chain(options)
            .chain(remote)
            .collect()
    }
}

impl MacEdges for MacWakeEdges {
    fn listening_here(&self) -> bool {
        let launchctl = ProcessLaunchctl::new(self.launchctl.clone());
        listening(&mut LaunchdUnits::new(launchctl, self.user_identifier))
    }

    fn seat_listening(&self) -> bool {
        self.seat_flag("listening")
    }

    fn avatar_host(&self) -> String {
        self.avatar_host.clone()
    }

    fn room_name(&self) -> Option<String> {
        self.room.clone()
    }

    fn at_the_avatar_host(&self, command: &[String], timeout_seconds: u64) -> Boundary<String> {
        let timeout = Duration::from_secs(timeout_seconds);
        let captured = run_captured(&self.ssh_command(command), &[], timeout);
        stdout_of(captured.map_err(|error| unavailable(&error))?)
    }

    fn open_here(&self) -> Boundary<bool> {
        let captured = run_captured(&self.open_command, &[], OPEN_TIMEOUT);
        Ok(captured.map_err(|error| unavailable(&error))?.succeeded())
    }

    fn clock(&self) -> f64 {
        self.clock.monotonic()
    }
}
