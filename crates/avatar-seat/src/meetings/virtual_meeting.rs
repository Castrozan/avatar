use crate::edges::Processes;
use crate::local::SeatFailure;
use crate::meetings::readiness::Readiness;
use crate::support::words::joined_words;

const ACTIVE_STATES: [&str; 3] = ["active", "activating", "reloading"];

fn unestablished() -> SeatFailure {
    SeatFailure::Runtime("Avatar resource state could not be established".to_owned())
}

fn refused() -> SeatFailure {
    let message = "Existing avatar resources must be left untouched; virtual-only start refused";
    SeatFailure::Runtime(message.to_owned())
}

fn is_active_command(units: &[String]) -> Vec<String> {
    let prefix = joined_words(&[&["systemctl", "--user", "is-active"]]);
    [prefix, units.to_vec()].concat()
}

pub fn unit_states(
    processes: &dyn Processes,
    units: &[String],
    timeout_seconds: u64,
) -> Result<Vec<String>, SeatFailure> {
    let observed = processes.run(&is_active_command(units), timeout_seconds, true)?;
    let established = matches!(observed.code, 0 | 3) && !observed.stdout.trim().is_empty();
    let lines = || observed.stdout.lines().map(str::to_owned).collect();
    established.then(lines).ok_or_else(unestablished)
}

pub fn require_fresh_runtime(
    processes: &dyn Processes,
    readiness: &dyn Readiness,
    units: &[String],
    timeout_seconds: u64,
) -> Result<(), SeatFailure> {
    let states = unit_states(processes, units, timeout_seconds)?;
    let busy = states
        .iter()
        .any(|state| ACTIVE_STATES.contains(&state.as_str()));
    if busy || readiness.audio_ready()? {
        Err(refused())
    } else {
        Ok(())
    }
}
