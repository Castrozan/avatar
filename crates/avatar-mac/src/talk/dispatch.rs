use avatar_listening::conversation_state::{ConversationPhase, decide};
use avatar_seat::talk::asks_for_the_remote_seat;

use super::HOST_OPTIONS;
use super::live::LiveOpening;
use super::perform::perform;
use super::ports::{HostTalk, MacTalkRoom};
use crate::failure::Failure;

fn carry_out_here(
    room: &mut impl MacTalkRoom,
    live: &mut impl LiveOpening,
    arguments: &[String],
) -> Result<i32, Failure> {
    let request = room.parse_request(arguments)?;
    let action = decide(room.phase(), request.command, None, None);
    Ok(perform(room, live, action, &request)?.unwrap_or(0))
}

fn belongs_to_the_host(room: &mut impl MacTalkRoom, arguments: &[String]) -> bool {
    let named = HOST_OPTIONS
        .iter()
        .any(|option| arguments.iter().any(|argument| argument == option));
    named || room.conversation_listening()
}

fn make_room_for_the_host(
    room: &mut impl MacTalkRoom,
    arguments: &[String],
) -> Result<(), Failure> {
    let opening = arguments.first().is_some_and(|verb| verb == "open");
    if opening && room.phase() == ConversationPhase::Open {
        return super::conversation::end_conversation(room);
    }
    Ok(())
}

fn opens_a_seat_away_from_this_mac(arguments: &[String]) -> bool {
    let opening = arguments.first().is_some_and(|verb| verb == "open");
    let seated = arguments.iter().any(|argument| argument == "--seat");
    opening && seated && !asks_for_the_remote_seat(arguments)
}

fn carry_out_at_the_host(
    room: &mut impl MacTalkRoom,
    host: &mut impl HostTalk,
    arguments: &[String],
) -> Result<i32, Failure> {
    make_room_for_the_host(room, arguments)?;
    let forwarded = host.with_the_agent_that_receives_live_tasks(arguments);
    let code = host.carry_out_at_host(&forwarded)?;
    if code == 0 && opens_a_seat_away_from_this_mac(arguments) {
        host.remember_the_conversation_at_the_host();
    }
    Ok(code)
}

pub fn carry_out<Room: MacTalkRoom>(
    room: &mut Room,
    host: &mut impl HostTalk,
    live: &mut impl LiveOpening,
    arguments: &[String],
) -> Result<i32, Failure> {
    if let Some(forwarded) = host.forwarded_from_this_machine(arguments) {
        return Ok(forwarded);
    }
    if belongs_to_the_host(room, arguments) {
        return carry_out_at_the_host(room, host, arguments);
    }
    if let Some(forwarded) = host.forwarded_to_the_conversation_opened_elsewhere(arguments) {
        return Ok(forwarded);
    }
    carry_out_here(room, live, arguments)
}

pub fn run<Room: MacTalkRoom>(
    room: &mut Room,
    host: &mut impl HostTalk,
    live: &mut impl LiveOpening,
    arguments: &[String],
) -> i32 {
    carry_out(room, host, live, arguments).unwrap_or_else(|failure| {
        room.print_error(&failure.message);
        1
    })
}
