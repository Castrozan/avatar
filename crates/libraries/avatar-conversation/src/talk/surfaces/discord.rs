use serde_json::Value;

use crate::talk::failure::TalkFailure;
use crate::talk::ports::exec::run_unchecked;
use crate::talk::ports::{Clock, Subprocess};

const DISCORD_VOICE: &str = "discord-voice";
const COMMAND_TIMEOUT_SECONDS: f64 = 30.0;
const JOIN_TIMEOUT_SECONDS: f64 = 75.0;
const STATUS_TIMEOUT_SECONDS: f64 = 5.0;
const JOIN_WAIT_SECONDS: f64 = 20.0;
const POLL_SECONDS: f64 = 0.5;
const COULD_NOT_JOIN: &str = "The avatar could not join a Discord call";
const JOIN_REFUSALS: [(&str, &str); 5] = [
    (
        "listener_not_in_allowed_voice_channel",
        "join a Discord voice channel first",
    ),
    (
        "bridge_not_ready",
        "the Discord bots are not connected yet, see discord-voice status",
    ),
    (
        "no_eligible_available_bot_for_listener",
        "no Discord bot is free to join your channel, see discord-voice status",
    ),
    ("bridge_not_running", "the Discord bridge is not running"),
    ("bridge_did_not_start", "the Discord bridge did not start"),
];

fn run_discord_voice(
    process: &mut impl Subprocess,
    subcommand: &str,
    timeout_seconds: f64,
) -> String {
    run_unchecked(process, &[DISCORD_VOICE, subcommand], Some(timeout_seconds))
        .map_or_else(|_| String::new(), |output| output.stdout)
}

fn read_status(process: &mut impl Subprocess) -> Option<Value> {
    let stdout = run_discord_voice(process, "status", STATUS_TIMEOUT_SECONDS);
    let status: Value = serde_json::from_str(&stdout).ok()?;
    status.as_object()?.contains_key("joined").then_some(status)
}

fn refusal_text(error: &Value) -> String {
    let name = error
        .as_str()
        .map_or_else(|| error.to_string(), str::to_owned);
    let known = JOIN_REFUSALS.iter().find(|(code, _)| *code == name);
    known.map_or_else(
        || format!("the Discord bridge refused with {name}"),
        |(_, text)| (*text).to_owned(),
    )
}

fn join_refusal(reply: Option<&Value>) -> Option<String> {
    let Some(Value::Object(fields)) = reply else {
        return Some("the Discord bridge did not answer".to_owned());
    };
    fields
        .get("error")
        .filter(|error| !error.is_null())
        .map(refusal_text)
}

pub fn in_call(process: &mut impl Subprocess) -> bool {
    read_status(process).is_some_and(|status| status["joined"] == Value::Bool(true))
}

pub fn listener_left_the_call(process: &mut impl Subprocess) -> bool {
    read_status(process).is_some_and(|status| {
        !(status["joined"] == Value::Bool(true)
            && status.get("listener_present") == Some(&Value::Bool(true)))
    })
}

pub fn join(boundary: &mut (impl Subprocess + Clock)) -> Result<(), TalkFailure> {
    let stdout = run_discord_voice(boundary, "join", JOIN_TIMEOUT_SECONDS);
    if let Some(refusal) = join_refusal(serde_json::from_str(&stdout).ok().as_ref()) {
        return Err(TalkFailure::new(format!("{COULD_NOT_JOIN}: {refusal}")));
    }
    let deadline = boundary.monotonic() + JOIN_WAIT_SECONDS;
    while !in_call(boundary) {
        wait_or_give_up(boundary, deadline)?;
    }
    Ok(())
}

fn wait_or_give_up(clock: &mut impl Clock, deadline: f64) -> Result<(), TalkFailure> {
    if clock.monotonic() > deadline {
        return Err(TalkFailure::new(format!(
            "{COULD_NOT_JOIN}: the Discord bridge did not join in time"
        )));
    }
    clock.sleep(POLL_SECONDS);
    Ok(())
}

pub fn leave(process: &mut impl Subprocess) {
    run_discord_voice(process, "leave", COMMAND_TIMEOUT_SECONDS);
}
