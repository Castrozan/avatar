mod addressee;
mod boundaries;
mod requests;

use crate::addressee::Addressee;
use crate::floor::{self, RemoteAgent};
use boundaries::{LiveSessions, ShownServants};
use requests::{HandoverRequest, field, parse, sessions, with_file};
use serde_json::{Value, json};
use std::path::PathBuf;

pub(super) fn handle(name: &str, request: &Value) -> Result<Value, String> {
    match name.strip_prefix("addressee.") {
        Some(operation) => addressee::handle(operation, request),
        None => floor_operation(name, request),
    }
}

fn floor_operation(name: &str, request: &Value) -> Result<Value, String> {
    match name {
        "claim" => claim(request),
        "may_speak" => may_speak(request),
        "first_claim" => first_claim(request),
        "give_the_room_to" => give_the_room_to(request),
        "hand_to_remote_agent" => hand_to_remote_agent(request),
        _ => Err(format!("unknown floor operation {name}")),
    }
}

fn claim(request: &Value) -> Result<Value, String> {
    let directory: PathBuf = field(request, "directory")?;
    let pane: String = field(request, "pane")?;
    let boundaries = LiveSessions {
        sessions: sessions(request)?,
        listening: field(request, "listening")?,
    };
    let holder = floor::claim(&directory, &pane, &boundaries).map_err(|error| error.to_string())?;
    with_file(&directory, json!({ "holder": holder }))
}

fn may_speak(request: &Value) -> Result<Value, String> {
    let holder: Option<Addressee> = field(request, "holder")?;
    let pane: String = field(request, "pane")?;
    Ok(json!(floor::may_speak(holder.as_ref(), &pane)))
}

fn first_claim(request: &Value) -> Result<Value, String> {
    let holder: Option<Addressee> = field(request, "holder")?;
    let pane: String = field(request, "pane")?;
    let claimer = floor::first_claim(&pane, &sessions(request)?, holder.as_ref());
    Ok(json!(claimer))
}

fn give_the_room_to(request: &Value) -> Result<Value, String> {
    let directory: PathBuf = field(request, "directory")?;
    let addressee: Addressee = field(request, "addressee")?;
    let servants = ShownServants::default();
    floor::give_the_room_to(&directory, &addressee, &servants)
        .map_err(|error| error.to_string())?;
    with_file(&directory, json!({ "shown": servants.into_shown() }))
}

fn hand_to_remote_agent(request: &Value) -> Result<Value, String> {
    let handover: HandoverRequest = parse(request)?;
    let servants = ShownServants::default();
    let handed = remote_handover(&handover, &servants)?;
    let shown = servants.into_shown();
    with_file(
        &handover.directory,
        json!({ "handed": handed, "shown": shown }),
    )
}

fn remote_handover(handover: &HandoverRequest, servants: &ShownServants) -> Result<bool, String> {
    let agent = RemoteAgent::from(&handover.agent);
    let servant_at = |_: &RemoteAgent| handover.servant.clone();
    floor::hand_to_remote_agent(
        &handover.directory,
        &agent,
        handover.heard_through,
        &servant_at,
        servants,
    )
    .map_err(|error| error.to_string())
}
