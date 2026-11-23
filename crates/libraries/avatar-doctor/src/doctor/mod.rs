mod command;
mod failure;
mod finding;
mod host;
mod mac;
mod page;
mod process;
mod seat;
mod wake;

pub use command::{CommandOutput, CommandRunner};
pub use failure::DoctorError;
pub use finding::{Finding, all_healthy, examined, rendered_json, rendered_text};
pub use page::{LocalPageProbe, PageProbe};
pub use process::{COMMAND_TIMEOUT, SystemCommandRunner};

#[must_use]
pub fn host_report(runner: &impl CommandRunner) -> Vec<Finding> {
    examined(host::host_findings(runner))
}

#[must_use]
pub fn seat_report(
    runner: &impl CommandRunner,
    probe: &impl PageProbe,
    host: &str,
    user_id: u32,
) -> Vec<Finding> {
    examined(seat::seat_findings(runner, probe, host, user_id))
}
