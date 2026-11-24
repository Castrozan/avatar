use crate::room_reach::{
    CALLER_HOLDS_INPUT_VARIABLE, ForwardedCommand, forwarding_plan, this_host,
};
use crate::talk::failure::TalkFailure;
use crate::talk::ports::exec::relay_unchecked;
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

pub(super) fn on_host(room: &str, command: &[String]) -> Vec<String> {
    let joined: Vec<String> = command
        .iter()
        .map(|argument| shell_quote(argument))
        .collect();
    let mut argv: Vec<String> = REACH_HOST.iter().map(|part| (*part).to_owned()).collect();
    argv.extend([room.to_owned(), joined.join(" ")]);
    argv
}

fn ending_with_its_caller(command: &[String]) -> Vec<String> {
    let held = format!("{CALLER_HOLDS_INPUT_VARIABLE}=1");
    [vec!["env".to_owned(), held], command.to_vec()].concat()
}

pub(super) fn plan_towards(
    environment: &impl Environment,
    room: Option<&str>,
    arguments: &[String],
) -> Option<ForwardedCommand> {
    let configured = environment.variable(HOST_VARIABLE);
    let host = this_host(configured.as_deref(), &environment.hostname());
    let pane = this_pane(environment);
    forwarding_plan(room, &host, arguments, pane.as_deref())
}

pub fn forwarded_from_this_machine(
    boundary: &mut (impl Subprocess + Environment + Console),
    arguments: &[String],
) -> Option<i32> {
    let room = boundary.variable(ROOM_VARIABLE);
    let plan = plan_towards(boundary, room.as_deref(), arguments)?;
    Some(relayed(boundary, &plan))
}

pub(super) fn relayed(boundary: &mut (impl Subprocess + Console), plan: &ForwardedCommand) -> i32 {
    let argv = on_host(&plan.room, &ending_with_its_caller(&plan.command));
    relay_unchecked(boundary, &argv, Some(LONGEST_SEAT_SECONDS))
        .unwrap_or_else(|failure| report_failure(boundary, &failure))
}

fn report_failure(console: &mut impl Console, failure: &TalkFailure) -> i32 {
    console.print_err(&format!("{failure}\n"));
    1
}
