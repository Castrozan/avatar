use std::time::Duration;

use avatar_conversation::talk::TalkFailure;
use avatar_conversation::talk::ports::Readiness;
use avatar_conversation::talk::ports::exec::run_unchecked;

use super::SystemdUnits;
use super::readiness::all_ready;
use crate::clock::monotonic_seconds;

const IS_FAILED_SECONDS: f64 = 10.0;
const FAILED_MESSAGE: &str = "An avatar service failed; inspect systemctl --user status avatar-*";

fn service_failed(units: &mut SystemdUnits, names: &[&str]) -> Result<bool, TalkFailure> {
    let mut argv = vec!["systemctl", "--user", "is-failed"];
    argv.extend(names);
    let output = run_unchecked(&mut units.process, &argv, Some(IS_FAILED_SECONDS))?;
    Ok(output.stdout.lines().any(|line| line == "failed"))
}

fn poll(
    units: &mut SystemdUnits,
    names: &[&str],
    checks: &[Readiness],
) -> Result<bool, TalkFailure> {
    if service_failed(units, names)? {
        return Err(TalkFailure::new(FAILED_MESSAGE));
    }
    all_ready(units, checks)
}

fn ready_before(
    units: &mut SystemdUnits,
    names: &[&str],
    checks: &[Readiness],
    deadline: f64,
) -> Result<bool, TalkFailure> {
    let mut ready = false;
    while !ready && monotonic_seconds() < deadline {
        ready = poll(units, names, checks)?;
        (!ready).then(|| pause(units));
    }
    Ok(ready)
}

fn pause(units: &SystemdUnits) {
    std::thread::sleep(Duration::from_secs_f64(units.timing.poll_seconds));
}

pub(super) fn wait_for_units(
    units: &mut SystemdUnits,
    names: &[&str],
    checks: &[Readiness],
) -> Result<(), TalkFailure> {
    let seconds = units.timing.timeout_seconds;
    if ready_before(units, names, checks, monotonic_seconds() + seconds)? {
        return Ok(());
    }
    let message = format!(
        "{} did not become ready within {seconds} seconds",
        names.join(", ")
    );
    Err(TalkFailure::new(message))
}
