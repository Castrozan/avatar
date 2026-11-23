use avatar_floor::quiet::Seat;

use super::bring_up::{BringUp, bring_up};
use super::channels::audio_command;
use super::targets::Target;
use super::{AUDIO_COMMAND_TIMEOUT_SECONDS, SYSTEMCTL_QUERY_TIMEOUT_SECONDS, StartEdges};
use crate::local::SeatFailure;
use crate::meetings::virtual_meeting::require_fresh_runtime;

const VIRTUAL_READY: &str = r#"{"ready": true, "owner": "avatar.target", "camera": "Avatar Cam", "microphone": "AvatarMicrophone", "physical_capture": false, "client_device_selection": "required"}"#;

fn fresh_units() -> Vec<String> {
    let targets = [Target::Meeting, Target::Conversation].map(|target| target.name().to_owned());
    let listener = vec!["avatar-listener.service".to_owned()];
    [targets.to_vec(), listener, Target::Meeting.units()].concat()
}

fn require_room(seat: Seat) -> Result<(), SeatFailure> {
    if seat == Seat::Room {
        return Ok(());
    }
    let message = "Virtual-only preparation requires the local meeting seat";
    Err(SeatFailure::Runtime(message.to_owned()))
}

fn require_fresh(edges: &StartEdges<'_>) -> Result<(), SeatFailure> {
    let timeout = SYSTEMCTL_QUERY_TIMEOUT_SECONDS;
    require_fresh_runtime(edges.processes, edges.readiness, &fresh_units(), timeout)
}

fn stop_audio(edges: &StartEdges<'_>) -> Result<(), SeatFailure> {
    let command = audio_command(&["stop"]);
    edges
        .processes
        .run(&command, AUDIO_COMMAND_TIMEOUT_SECONDS, false)?;
    Ok(())
}

fn bring_up_or_stop_audio(edges: &StartEdges<'_>) -> Result<(), SeatFailure> {
    let brought = bring_up(edges, &BringUp::meeting());
    if brought.is_err() {
        stop_audio(edges)?;
    }
    brought
}

pub(super) fn start_virtual_only(edges: &StartEdges<'_>, seat: Seat) -> Result<(), SeatFailure> {
    require_room(seat)?;
    require_fresh(edges)?;
    bring_up_or_stop_audio(edges)?;
    edges.output.line(VIRTUAL_READY);
    Ok(())
}
