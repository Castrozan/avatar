use avatar_listening::conversation_state::ConversationPhase;
use serde_json::Value;

use super::reach::{on_host, plan_towards, relayed};
use crate::room_reach::ForwardedCommand;
use crate::talk::ports::exec::run_unchecked;
use crate::talk::ports::{Console, Environment, Files, Subprocess, Units};
use crate::talk::state::elsewhere::{
    forget_the_conversation_elsewhere, machine_holding_the_conversation,
};
use crate::talk::state::phase::phase;

const OPEN_VERB: &str = "open";
const CLOSE_VERB: &str = "close";
const STATUS_VERB: &str = "status";
const LOOK_SECONDS: f64 = 20.0;

fn verb_is(arguments: &[String], verb: &str) -> bool {
    arguments.first().is_some_and(|first| first == verb)
}

fn plan_for_elsewhere(
    boundary: &mut (impl Environment + Files + Units),
    arguments: &[String],
) -> Option<ForwardedCommand> {
    if verb_is(arguments, OPEN_VERB) {
        return None;
    }
    let machine = machine_holding_the_conversation(boundary)?;
    if phase(boundary) != ConversationPhase::Closed {
        return None;
    }
    plan_towards(boundary, Some(&machine), arguments)
}

pub fn forwarded_to_the_conversation_opened_elsewhere(
    boundary: &mut (impl Subprocess + Environment + Console + Files + Units),
    arguments: &[String],
) -> Option<i32> {
    let plan = plan_for_elsewhere(boundary, arguments)?;
    let code = relayed(boundary, &plan);
    settle_the_record(boundary, &plan.room, arguments, code);
    Some(code)
}

fn settle_the_record(
    boundary: &mut (impl Subprocess + Environment + Console + Files),
    machine: &str,
    arguments: &[String],
    code: i32,
) {
    if verb_is(arguments, CLOSE_VERB) && code == 0 {
        return forget_the_conversation_elsewhere(boundary);
    }
    let unsure = code != 0 || verb_is(arguments, STATUS_VERB);
    if unsure && closed_there(boundary, machine) {
        forget_the_conversation_elsewhere(boundary);
        boundary.print_err(&closed_notice(machine));
    }
}

fn closed_notice(machine: &str) -> String {
    format!(
        "The conversation this machine opened on {machine} is closed; avatar-talk runs here again\n"
    )
}

fn closed_there(boundary: &mut (impl Subprocess + Environment), machine: &str) -> bool {
    let status = [STATUS_VERB.to_owned()];
    let Some(plan) = plan_towards(boundary, Some(machine), &status) else {
        return false;
    };
    let argv = on_host(&plan.room, &plan.command);
    let looked = run_unchecked(boundary, &argv, Some(LOOK_SECONDS));
    looked.is_ok_and(|output| output.status == 0 && reports_closed(&output.stdout))
}

fn reports_closed(status_output: &str) -> bool {
    let last_line = status_output.trim().lines().next_back().unwrap_or_default();
    let report: Option<Value> = serde_json::from_str(last_line).ok();
    report.is_some_and(|status| status["open"] == Value::Bool(false))
}
