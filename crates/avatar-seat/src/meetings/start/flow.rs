use avatar_floor::quiet::Seat;

use super::StartEdges;
use super::bring_up::{BringUp, bring_up};
use super::channels::{open_call_channels, run_audio};
use super::virtual_only::start_virtual_only;
use crate::local::SeatFailure;

const REMOTE_READY: &str = "Avatar ready with her voice and the microphone at the remote seat";
const ROOM_READY: &str = "Avatar ready with her voice and the default microphone open. Check levels with avatar-audio status. Switch video with avatar-command mode";

fn finish_remote(edges: &StartEdges<'_>, seat: Seat) -> Result<(), SeatFailure> {
    edges.seats.prepare(seat)?;
    open_call_channels(edges)?;
    edges.output.line(REMOTE_READY);
    Ok(())
}

fn finish_room(edges: &StartEdges<'_>) -> Result<(), SeatFailure> {
    run_audio(edges, &["open"])?;
    edges.output.line(ROOM_READY);
    Ok(())
}

pub fn start(edges: &StartEdges<'_>, seat: Seat, virtual_only: bool) -> Result<(), SeatFailure> {
    if virtual_only {
        return start_virtual_only(edges, seat);
    }
    bring_up(edges, &BringUp::meeting())?;
    if seat == Seat::Remote {
        return finish_remote(edges, seat);
    }
    finish_room(edges)
}
