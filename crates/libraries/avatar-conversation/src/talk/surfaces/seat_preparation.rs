use avatar_floor::quiet::Seat;

use crate::talk::failure::TalkFailure;
use crate::talk::ports::exec::run_checked;
use crate::talk::ports::{Clock, Subprocess, Units};
use crate::talk::surfaces::discord;

const REMOTE_SEAT_SERVICE: &str = "avatar-remote-seat.service";
const REMOTE_SEAT_WAIT_SECONDS: f64 = 15.0;
const POLL_SECONDS: f64 = 0.25;
const SYSTEMCTL_TIMEOUT_SECONDS: f64 = 120.0;

pub fn prepare(
    boundary: &mut (impl Subprocess + Clock + Units),
    seat: Seat,
) -> Result<(), TalkFailure> {
    match seat {
        Seat::Discord => discord::join(boundary),
        Seat::Remote => prepare_remote_seat(boundary),
        Seat::Room => Ok(()),
    }
}

fn wait_or_give_up(boundary: &mut (impl Clock + Units), deadline: f64) -> Result<(), TalkFailure> {
    if boundary.monotonic() > deadline {
        return Err(TalkFailure::new("The remote seat did not start listening"));
    }
    boundary.sleep(POLL_SECONDS);
    Ok(())
}

fn prepare_remote_seat(
    boundary: &mut (impl Subprocess + Clock + Units),
) -> Result<(), TalkFailure> {
    let argv = ["systemctl", "--user", "start", REMOTE_SEAT_SERVICE];
    run_checked(boundary, &argv, Some(SYSTEMCTL_TIMEOUT_SECONDS))?;
    let deadline = boundary.monotonic() + REMOTE_SEAT_WAIT_SECONDS;
    while !boundary.remote_seat_listening() {
        wait_or_give_up(boundary, deadline)?;
    }
    Ok(())
}
