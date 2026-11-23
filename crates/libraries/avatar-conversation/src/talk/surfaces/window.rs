use avatar_floor::quiet::Seat;

use crate::talk::failure::TalkFailure;
use crate::talk::ports::exec::run_checked;
use crate::talk::ports::{Clock, Files, Readiness, Subprocess, Units};
use crate::talk::state::seat::current;
use crate::talk::state::{MEETING_TARGET, WINDOW_TARGET};
use crate::talk::surfaces::desktop;

const COMMAND_TIMEOUT_SECONDS: f64 = 120.0;
const WINDOW_UNITS: [&str; 3] = ["avatar-control-server", "avatar-renderer", "avatar-browser"];

fn set_canvas_presence(process: &mut impl Subprocess, presence: &str) -> Result<(), TalkFailure> {
    let argv = ["avatar-command", "presence", presence];
    run_checked(process, &argv, Some(COMMAND_TIMEOUT_SECONDS)).map(|_| ())
}

pub fn hide_everywhere(process: &mut (impl Subprocess + Clock)) -> Result<(), TalkFailure> {
    set_canvas_presence(process, "hidden")?;
    desktop::hide(process)
}

pub fn reveal_window(
    boundary: &mut (impl Subprocess + Clock + Units + Files),
) -> Result<(), TalkFailure> {
    boundary.wait_until_ready(&WINDOW_UNITS, &[Readiness::Renderer])?;
    if current(boundary)? == Seat::Room {
        desktop::show(boundary)?;
    }
    set_canvas_presence(boundary, "shown")
}

pub fn show_window(units: &mut impl Units) -> Result<(), TalkFailure> {
    units.bring_up(WINDOW_TARGET, &[Readiness::Renderer])
}

pub fn hide_window(boundary: &mut (impl Subprocess + Clock + Units)) -> Result<(), TalkFailure> {
    let argv = ["systemctl", "--user", "stop", WINDOW_TARGET];
    run_checked(boundary, &argv, Some(COMMAND_TIMEOUT_SECONDS))?;
    if boundary.unit_active(MEETING_TARGET) {
        return hide_everywhere(boundary);
    }
    Ok(())
}
