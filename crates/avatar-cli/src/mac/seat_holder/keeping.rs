use avatar_seat::edges::EdgeFailure;
use avatar_seat::holder::{HoldEdges, MeetingSeat, TunnelProcess, hold_seat};
use avatar_seat::local::SeatConfig;
use avatar_seat::meetings::camera::{FileSystemStore, ObsCamera};
use avatar_seat::talk::LONGEST_SEAT_SECONDS;

use super::super::room::camera_of;

pub(super) fn longest_seat_seconds() -> u32 {
    u32::try_from(LONGEST_SEAT_SECONDS).unwrap_or(u32::MAX)
}

pub(super) fn hold_meeting(
    config: &SeatConfig,
    tunnel: &mut impl TunnelProcess,
    edges: &HoldEdges<'_>,
) -> Result<(), EdgeFailure> {
    let camera_config = camera_of(config);
    let camera = ObsCamera {
        config: &camera_config,
        processes: edges.local.processes,
        clock: edges.clock,
        store: &FileSystemStore,
    };
    let mut seat = MeetingSeat::new(camera, edges.reporter);
    hold_seat(&mut seat, tunnel, edges, longest_seat_seconds())
}
