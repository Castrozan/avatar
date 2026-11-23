use std::io;

use super::{LaunchctlPrint, agent_target, agents_for, unit_active};
use crate::user_units::{Clock, UnitEffectError};

pub const STOP_WAIT_SECONDS: f64 = 10.0;
pub const POLL_SECONDS: f64 = 0.1;

pub trait LaunchctlControl {
    fn kickstart(&mut self, target: &str) -> io::Result<()>;
    fn terminate(&mut self, target: &str);
}

fn standing_agent(unit: &str) -> Result<&'static str, UnitEffectError> {
    let agents = agents_for(unit);
    agents
        .first()
        .copied()
        .ok_or_else(|| UnitEffectError::UnknownUnit(unit.to_owned()))
}

fn service_target(user_identifier: u32, unit: &str) -> Result<String, UnitEffectError> {
    Ok(agent_target(user_identifier, standing_agent(unit)?))
}

fn ensure_before(now: f64, deadline: f64, unit: &str) -> Result<(), UnitEffectError> {
    if now > deadline {
        return Err(UnitEffectError::DidNotStop(
            standing_agent(unit)?.to_owned(),
        ));
    }
    Ok(())
}

pub fn start(
    launchctl: &mut impl LaunchctlControl,
    user_identifier: u32,
    unit: &str,
) -> Result<(), UnitEffectError> {
    let target = service_target(user_identifier, unit)?;
    Ok(launchctl.kickstart(&target)?)
}

pub fn stop<Launchctl: LaunchctlPrint + LaunchctlControl>(
    launchctl: &mut Launchctl,
    clock: &mut impl Clock,
    user_identifier: u32,
    unit: &str,
) -> Result<(), UnitEffectError> {
    stop_within(launchctl, clock, user_identifier, unit, STOP_WAIT_SECONDS)
}

pub fn stop_within<Launchctl: LaunchctlPrint + LaunchctlControl>(
    launchctl: &mut Launchctl,
    clock: &mut impl Clock,
    user_identifier: u32,
    unit: &str,
    wait_seconds: f64,
) -> Result<(), UnitEffectError> {
    let target = service_target(user_identifier, unit)?;
    launchctl.terminate(&target);
    let deadline = clock.monotonic_seconds() + wait_seconds;
    while unit_active(launchctl, user_identifier, unit) {
        ensure_before(clock.monotonic_seconds(), deadline, unit)?;
        clock.sleep_seconds(POLL_SECONDS);
    }
    Ok(())
}
