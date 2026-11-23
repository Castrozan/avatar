use super::{SYSTEMCTL_QUERY_TIMEOUT_SECONDS, StartEdges};
use crate::local::{POLL_SECONDS, SeatFailure};
use crate::meetings::readiness::{Check, check_ready};
use crate::support::words::joined_words;

const SERVICE_FAILED: &str = "An avatar service failed; inspect systemctl --user status avatar-*";

pub struct ReadinessWait {
    pub units: Vec<String>,
    pub checks: Vec<Check>,
    pub timeout_seconds: u32,
}

pub fn service_failed(edges: &StartEdges<'_>, units: &[String]) -> Result<bool, SeatFailure> {
    let prefix = joined_words(&[&["systemctl", "--user", "is-failed"]]);
    let command = [prefix, units.to_vec()].concat();
    let timeout = SYSTEMCTL_QUERY_TIMEOUT_SECONDS;
    let failed = edges.processes.run(&command, timeout, true)?;
    Ok(failed.stdout.lines().any(|state| state == "failed"))
}

fn all_ready(edges: &StartEdges<'_>, checks: &[Check]) -> Result<bool, SeatFailure> {
    let Some((check, rest)) = checks.split_first() else {
        return Ok(true);
    };
    Ok(check_ready(edges.readiness, *check)? && all_ready(edges, rest)?)
}

fn settled(edges: &StartEdges<'_>, wait: &ReadinessWait) -> Result<bool, SeatFailure> {
    if service_failed(edges, &wait.units)? {
        return Err(SeatFailure::Runtime(SERVICE_FAILED.to_owned()));
    }
    all_ready(edges, &wait.checks)
}

fn settled_or_sleep(edges: &StartEdges<'_>, wait: &ReadinessWait) -> Result<bool, SeatFailure> {
    let ready = settled(edges, wait)?;
    if !ready {
        edges.clock.sleep(POLL_SECONDS);
    }
    Ok(ready)
}

fn never_ready(wait: &ReadinessWait) -> SeatFailure {
    let units = wait.units.join(", ");
    let seconds = wait.timeout_seconds;
    SeatFailure::Timeout(format!(
        "{units} did not become ready within {seconds} seconds"
    ))
}

pub fn wait_until_ready(edges: &StartEdges<'_>, wait: &ReadinessWait) -> Result<(), SeatFailure> {
    let deadline = edges.clock.monotonic() + f64::from(wait.timeout_seconds);
    let mut ready = false;
    while !ready && edges.clock.monotonic() < deadline {
        ready = settled_or_sleep(edges, wait)?;
    }
    ready.then_some(()).ok_or_else(|| never_ready(wait))
}
