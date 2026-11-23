use std::time::Duration;

use crate::process::loopback_get;

const SEAT_TIMEOUT: Duration = Duration::from_secs(1);

pub(super) fn tell_seat_levels_changed(seat_port: u16) {
    let _ = loopback_get(seat_port, "/levels", SEAT_TIMEOUT);
}
