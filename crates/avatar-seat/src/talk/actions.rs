use super::REMOTE_SEAT;
use super::arguments::{
    asks_for_the_remote_seat, opens_a_seated_conversation, reported_seat, with_the_remote_seat,
};
use super::forward::{forward, forward_command};
use super::presence::user_is_here;
use crate::edges::EdgeFailure;
use crate::local::{LocalSeat, SeatFailure, hide_window, show_window};

const OPEN_TIMEOUT_SECONDS: u64 = 120;

fn verb_is(arguments: &[String], verb: &str) -> bool {
    arguments.first().is_some_and(|first| first == verb)
}

pub(crate) fn carry_out(
    local: &LocalSeat<'_>,
    arguments: &[String],
    stdout: &mut String,
) -> Result<i32, SeatFailure> {
    if opens_here(local, arguments)? {
        return open_here(local, arguments, stdout);
    }
    if verb_is(arguments, "close") {
        return close_everywhere(local, arguments);
    }
    if moves_the_window_of_a_seat_held_here(local, arguments) {
        return move_window_here(local, arguments);
    }
    Ok(forward(local, arguments)?)
}

fn opens_here(local: &LocalSeat<'_>, arguments: &[String]) -> Result<bool, EdgeFailure> {
    if !verb_is(arguments, "open") {
        return Ok(false);
    }
    if asks_for_the_remote_seat(arguments) {
        return Ok(true);
    }
    Ok(opens_a_seated_conversation(arguments) && user_is_here(local)?)
}

fn moves_the_window_of_a_seat_held_here(local: &LocalSeat<'_>, arguments: &[String]) -> bool {
    let is_window_verb = matches!(arguments, [verb] if verb == "show" || verb == "hide");
    is_window_verb && local.attached()
}

fn open_here(
    local: &LocalSeat<'_>,
    arguments: &[String],
    stdout: &mut String,
) -> Result<i32, SeatFailure> {
    let command = forward_command(local, &with_the_remote_seat(arguments));
    let opened = local.processes.run(&command, OPEN_TIMEOUT_SECONDS, true)?;
    stdout.push_str(&opened.stdout);
    if opened.code != 0 {
        return Ok(opened.code);
    }
    take_the_seat_if_remote(local, &opened.stdout)?;
    Ok(0)
}

fn take_the_seat_if_remote(local: &LocalSeat<'_>, status_output: &str) -> Result<(), SeatFailure> {
    let remote = reported_seat(status_output).is_some_and(|seat| seat == REMOTE_SEAT);
    if remote && !seat_held_here(local)? {
        local.take("")?;
    }
    Ok(())
}

fn seat_held_here(local: &LocalSeat<'_>) -> Result<bool, EdgeFailure> {
    Ok(local.attached() || local.meeting_seat_kept_here()?)
}

fn close_everywhere(local: &LocalSeat<'_>, arguments: &[String]) -> Result<i32, SeatFailure> {
    let code = forward(local, arguments)?;
    if !(local.meeting_held() || local.meeting_seat_kept_here()?) {
        local.leave()?;
    }
    Ok(code)
}

fn move_window_here(local: &LocalSeat<'_>, arguments: &[String]) -> Result<i32, SeatFailure> {
    let code = forward(local, arguments)?;
    if code == 0 {
        move_window(local, verb_is(arguments, "show"))?;
    }
    Ok(code)
}

fn move_window(local: &LocalSeat<'_>, showing: bool) -> Result<(), EdgeFailure> {
    if showing {
        show_window(local)
    } else {
        hide_window(local)
    }
}
