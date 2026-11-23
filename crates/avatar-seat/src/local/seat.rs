use serde_json::Value;

use super::COMMAND_TIMEOUT_SECONDS;
use super::config::{SeatConfig, tunnel_forward};
use super::failure::SeatFailure;
use crate::edges::{Clock, EdgeFailure, Processes, SeatProbe};
use crate::support::truth::python_truthy;

pub(crate) const TERMINATE_SIGNAL: &str = "TERM";
pub(crate) const HOLDER_COMMAND_LINE: &str = "mac seat-holder";
const MEETING_HOLDER_PATTERN: &str = r"mac\ seat-holder\ \?meeting";

pub struct LocalSeat<'edges> {
    pub config: &'edges SeatConfig,
    pub processes: &'edges dyn Processes,
    pub clock: &'edges dyn Clock,
    pub probe: &'edges dyn SeatProbe,
}

fn flag(health: Option<Value>, key: &str) -> bool {
    let found = health.and_then(|body| body.get(key).cloned());
    found.is_some_and(|value| python_truthy(&value))
}

fn words(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|part| (*part).to_owned()).collect()
}

fn holder_command(config: &SeatConfig, query: &str) -> Vec<String> {
    words(&[&config.holder_program, "mac", "seat-holder", query])
}

pub(crate) fn end_matching(
    processes: &dyn Processes,
    signal: &str,
    command_line: &str,
) -> Result<(), EdgeFailure> {
    let flag = format!("-{signal}");
    let command = words(&["/usr/bin/pkill", &flag, "-f", "--", command_line]);
    processes
        .run(&command, COMMAND_TIMEOUT_SECONDS, false)
        .map(drop)
}

impl<'edges> LocalSeat<'edges> {
    #[must_use]
    pub fn new(
        config: &'edges SeatConfig,
        processes: &'edges dyn Processes,
        clock: &'edges dyn Clock,
        probe: &'edges dyn SeatProbe,
    ) -> Self {
        Self {
            config,
            processes,
            clock,
            probe,
        }
    }
}

impl LocalSeat<'_> {
    #[must_use]
    pub fn attached(&self) -> bool {
        flag(self.probe.health(), "attached")
    }

    #[must_use]
    pub fn conversation_listening(&self) -> bool {
        flag(self.probe.health(), "listening")
    }

    #[must_use]
    pub fn meeting_held(&self) -> bool {
        flag(self.probe.health(), "meeting")
    }

    pub fn meeting_seat_kept_here(&self) -> Result<bool, EdgeFailure> {
        let command = words(&["/usr/bin/pgrep", "-f", "--", MEETING_HOLDER_PATTERN]);
        let found = self
            .processes
            .run(&command, COMMAND_TIMEOUT_SECONDS, true)?;
        Ok(found.code == 0)
    }

    pub fn end_processes_matching(&self, command_line: &str) -> Result<(), EdgeFailure> {
        end_matching(self.processes, TERMINATE_SIGNAL, command_line)
    }

    pub fn close_browsers(&self) -> Result<(), EdgeFailure> {
        self.end_processes_matching(&self.config.seat_profile_flag())?;
        self.end_processes_matching(&self.config.window_profile_flag())
    }

    pub fn leave(&self) -> Result<(), EdgeFailure> {
        self.end_processes_matching(HOLDER_COMMAND_LINE)?;
        self.end_processes_matching(&tunnel_forward())?;
        self.close_browsers()
    }

    pub fn take(&self, query: &str) -> Result<(), SeatFailure> {
        self.leave()?;
        let outcome = self.hold_and_open(query);
        outcome.or_else(|failure| self.leave().map_err(SeatFailure::from).and(Err(failure)))
    }

    fn hold_and_open(&self, query: &str) -> Result<(), SeatFailure> {
        self.processes
            .spawn_detached(&holder_command(self.config, query))?;
        self.wait_for_tunnel()?;
        self.open_browser(query)?;
        self.wait_until_attached()
    }
}
