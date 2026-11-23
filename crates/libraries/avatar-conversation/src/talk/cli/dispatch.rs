use avatar_listening::conversation_state::{ConversationAction, command_of, decide};

use super::actions::perform_action;
use super::arguments::{TalkArguments, parse_talk_arguments};
use super::reach::forwarded_from_this_machine;
use crate::talk::failure::TalkFailure;
use crate::talk::ports::{Boundaries, Console};
use crate::talk::state::phase::phase;
use crate::talk::state::seat::{configured, seat_name};
use crate::talk::state::this_pane;

fn decision_for(
    boundary: &mut impl Boundaries,
    arguments: &TalkArguments,
) -> Result<ConversationAction, TalkFailure> {
    let current = phase(boundary);
    let open = arguments.open();
    let command = command_of(open.live, arguments.command.conversation_command());
    let open_seat = open.seat.map(|_| configured(boundary)).transpose()?;
    Ok(decide(
        current,
        command,
        open.seat.map(seat_name),
        open_seat.map(seat_name),
    ))
}

fn report_parse_failure(console: &mut impl Console, error: &clap::Error) -> i32 {
    let rendered = error.render().to_string();
    if error.use_stderr() {
        console.print_err(&rendered);
    } else {
        console.print_out(&rendered);
    }
    error.exit_code()
}

fn report_failure(console: &mut impl Console, failure: &TalkFailure) -> i32 {
    console.print_err(&format!("{failure}\n"));
    1
}

fn perform(boundary: &mut impl Boundaries, arguments: &TalkArguments) -> Result<i32, TalkFailure> {
    let action = decision_for(boundary, arguments)?;
    perform_action(boundary, action, arguments)
}

pub fn run(boundary: &mut impl Boundaries, argv: &[String]) -> i32 {
    if let Some(code) = forwarded_from_this_machine(boundary, argv) {
        return code;
    }
    let arguments = match parse_talk_arguments(argv) {
        Ok(arguments) => arguments,
        Err(error) => return report_parse_failure(boundary, &error),
    };
    let pane = this_pane(boundary);
    boundary.report_talk(arguments.command.name(), pane.as_deref());
    perform(boundary, &arguments).unwrap_or_else(|failure| report_failure(boundary, &failure))
}
