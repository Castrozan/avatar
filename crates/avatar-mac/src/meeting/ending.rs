use super::ports::MeetingRoom;
use crate::failure::Failure;
use crate::stage::strike;

fn run_step(
    first: Option<Failure>,
    outcome: Result<(), Failure>,
) -> Result<Option<Failure>, Failure> {
    match outcome {
        Err(failure) if !failure.is_cleanup() => Err(failure),
        Err(failure) => Ok(first.or(Some(failure))),
        Ok(()) => Ok(first),
    }
}

pub fn end(room: &mut impl MeetingRoom) -> Result<(), Failure> {
    let first = run_step(None, room.restore_defaults())?;
    let first = run_step(first, room.stop_listening())?;
    let first = run_step(first, room.erase_session_records())?;
    let first = run_step(first, room.stop_camera())?;
    let first = run_step(first, room.close_browser())?;
    let first = run_step(first, strike(room))?;
    first.map_or(Ok(()), Err)
}

pub(super) fn after_cleanup(room: &mut impl MeetingRoom, failure: Failure) -> Failure {
    match end(room) {
        Err(cleanup) if !cleanup.is_cleanup() => cleanup,
        _ => failure,
    }
}
