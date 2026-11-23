use serde_json::Value;

use super::POLL_SECONDS;
use super::failure::SeatFailure;
use super::seat::LocalSeat;
use crate::support::truth::python_truthy;

const TUNNEL_WAIT_SECONDS: f64 = 15.0;
const ATTACH_WAIT_SECONDS: f64 = 20.0;

fn refuse_on_failure(seat: &Value) -> Result<(), SeatFailure> {
    let failure = seat.get("failure").filter(|reason| python_truthy(reason));
    let Some(failure) = failure else {
        return Ok(());
    };
    let reason = failure
        .as_str()
        .map_or_else(|| failure.to_string(), str::to_owned);
    Err(SeatFailure::Runtime(format!(
        "This machine could not take the seat: {reason}"
    )))
}

fn is_attached(seat: &Value) -> bool {
    seat.get("attached").is_some_and(python_truthy)
}

fn refuse_after(local: &LocalSeat<'_>, deadline: f64, message: &str) -> Result<(), SeatFailure> {
    if local.clock.monotonic() > deadline {
        return Err(SeatFailure::Timeout(message.to_owned()));
    }
    Ok(())
}

fn wait_for_tunnel(local: &LocalSeat<'_>) -> Result<(), SeatFailure> {
    let deadline = local.clock.monotonic() + TUNNEL_WAIT_SECONDS;
    while local.probe.health().is_none() {
        refuse_after(local, deadline, "The tunnel to the avatar did not open")?;
        local.clock.sleep(POLL_SECONDS);
    }
    Ok(())
}

fn attach_settled(local: &LocalSeat<'_>, seat: &Value) -> Result<bool, SeatFailure> {
    if is_attached(seat) {
        return Ok(true);
    }
    refuse_on_failure(seat)?;
    local.clock.sleep(POLL_SECONDS);
    Ok(false)
}

fn never_attached(local: &LocalSeat<'_>) -> SeatFailure {
    let browser = local.config.browser_name();
    let message =
        format!("The seat page did not attach; check that {browser} may use the microphone");
    SeatFailure::Timeout(message)
}

fn wait_until_attached(local: &LocalSeat<'_>) -> Result<(), SeatFailure> {
    let deadline = local.clock.monotonic() + ATTACH_WAIT_SECONDS;
    let mut attached = false;
    while !attached && local.clock.monotonic() < deadline {
        let seat = local.probe.health().unwrap_or(Value::Null);
        attached = attach_settled(local, &seat)?;
    }
    attached.then_some(()).ok_or_else(|| never_attached(local))
}

impl LocalSeat<'_> {
    pub(crate) fn wait_for_tunnel(&self) -> Result<(), SeatFailure> {
        wait_for_tunnel(self)
    }

    pub(crate) fn wait_until_attached(&self) -> Result<(), SeatFailure> {
        wait_until_attached(self)
    }
}
