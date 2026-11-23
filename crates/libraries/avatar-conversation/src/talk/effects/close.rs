use avatar_floor::quiet::Seat;
use avatar_holder::default_holder::DefaultHolder;
use avatar_listening::conversation_addressee::Addressee;
use avatar_listening::conversation_state::ConversationPhase;

use super::COMMAND_TIMEOUT_SECONDS;
use crate::talk::effects::default_holder::{floor_returns_to, take_the_floor};
use crate::talk::effects::report_lines::{closed_line, floor_returned_line, room_held_by_line};
use crate::talk::failure::TalkFailure;
use crate::talk::ports::exec::{run_checked, run_unchecked};
use crate::talk::ports::{Boundaries, Console, Files};
use crate::talk::state::addressee::holder_other_than;
use crate::talk::state::phase::phase;
use crate::talk::state::seat::current as current_seat;
use crate::talk::state::{CONVERSATION_TARGET, MEETING_TARGET, this_pane};
use crate::talk::surfaces::discord;
use crate::talk::surfaces::window::hide_everywhere;

fn silence_and_leave(boundary: &mut impl Boundaries, seat: Seat) -> Result<(), TalkFailure> {
    let argv = ["avatar-command", "stop-speaking"];
    run_unchecked(boundary, &argv, Some(COMMAND_TIMEOUT_SECONDS))?;
    if seat == Seat::Discord {
        discord::leave(boundary);
    }
    Ok(())
}

pub fn close_conversation(boundary: &mut impl Boundaries) -> Result<(), TalkFailure> {
    let seat = current_seat(boundary)?;
    silence_and_leave(boundary, seat)?;
    let stop = ["systemctl", "--user", "stop", CONVERSATION_TARGET];
    run_checked(boundary, &stop, Some(COMMAND_TIMEOUT_SECONDS))?;
    boundary.erase_session_records()?;
    stop_the_avatar(boundary)?;
    boundary.print_out(&format!("{}\n", closed_line()));
    Ok(())
}

fn stop_the_avatar(boundary: &mut impl Boundaries) -> Result<(), TalkFailure> {
    if boundary.unit_active(MEETING_TARGET) {
        return hide_everywhere(boundary);
    }
    run_checked(boundary, &["avatar-stop"], Some(COMMAND_TIMEOUT_SECONDS)).map(|_| ())
}

pub fn leave_meeting(console: &mut impl Console) -> Result<(), TalkFailure> {
    console.print_out(&format!("{}\n", closed_line()));
    Ok(())
}

pub fn close_or_return_the_floor(boundary: &mut impl Boundaries) -> Result<(), TalkFailure> {
    let pane = this_pane(boundary);
    let phase = phase(boundary);
    if let Some(holder) = other_holder(boundary, phase, pane.as_deref())? {
        boundary.print_out(&format!("{}\n", room_held_by_line(&holder.servant)));
        return Ok(());
    }
    match returning_holder(boundary, phase, pane.as_deref())? {
        Some(holder) => return_the_floor(boundary, &holder),
        None => close_conversation(boundary),
    }
}

fn other_holder(
    files: &mut impl Files,
    phase: ConversationPhase,
    pane: Option<&str>,
) -> Result<Option<Addressee>, TalkFailure> {
    if matches!(phase, ConversationPhase::Open | ConversationPhase::Live) {
        return holder_other_than(files, pane);
    }
    Ok(None)
}

fn returning_holder(
    files: &mut impl Files,
    phase: ConversationPhase,
    pane: Option<&str>,
) -> Result<Option<DefaultHolder>, TalkFailure> {
    if phase == ConversationPhase::Open {
        return floor_returns_to(files, pane);
    }
    Ok(None)
}

fn return_the_floor(
    boundary: &mut impl Boundaries,
    holder: &DefaultHolder,
) -> Result<(), TalkFailure> {
    take_the_floor(boundary, holder)?;
    boundary.print_out(&format!("{}\n", floor_returned_line(&holder.servant())));
    Ok(())
}
