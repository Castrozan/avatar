use crate::room_reach::forwarding_plan;
use crate::room_reach::this_host;
use crate::talk::failure::TalkFailure;
use crate::talk::ports::exec::run_unchecked;
use crate::talk::ports::{Console, Environment, Subprocess};
use crate::talk::state::this_pane;

const ROOM_VARIABLE: &str = "AVATAR_ROOM";
const HOST_VARIABLE: &str = "AVATAR_AGENT_HOST";
const LONGEST_SEAT_SECONDS: f64 = 43_200.0;
const REACH_HOST: [&str; 5] = ["ssh", "-o", "BatchMode=yes", "-o", "ConnectTimeout=10"];

fn shell_quote(argument: &str) -> String {
    let safe =
        |character: char| character.is_ascii_alphanumeric() || "_@%+=:,./-".contains(character);
    if !argument.is_empty() && argument.chars().all(safe) {
        return argument.to_owned();
    }
    format!("'{}'", argument.replace('\'', "'\"'\"'"))
}

fn on_host(room: &str, command: &[String]) -> Vec<String> {
    let joined: Vec<String> = command
        .iter()
        .map(|argument| shell_quote(argument))
        .collect();
    let mut argv: Vec<String> = REACH_HOST.iter().map(|part| (*part).to_owned()).collect();
    argv.extend([room.to_owned(), joined.join(" ")]);
    argv
}

pub(super) fn forwarded_from_this_machine(
    boundary: &mut (impl Subprocess + Environment + Console),
    arguments: &[String],
) -> Option<i32> {
    let room = boundary.variable(ROOM_VARIABLE);
    let configured = boundary.variable(HOST_VARIABLE);
    let host = this_host(configured.as_deref(), &boundary.hostname());
    let pane = this_pane(boundary);
    let plan = forwarding_plan(room.as_deref(), &host, arguments, pane.as_deref())?;
    Some(forward(boundary, &on_host(&plan.room, &plan.command)))
}

fn forward(boundary: &mut (impl Subprocess + Console), argv: &[String]) -> i32 {
    match run_unchecked(boundary, argv, Some(LONGEST_SEAT_SECONDS)) {
        Ok(output) => output.status,
        Err(failure) => report_failure(boundary, &failure),
    }
}

fn report_failure(console: &mut impl Console, failure: &TalkFailure) -> i32 {
    console.print_err(&format!("{failure}\n"));
    1
}
