use crate::claim::{claim, claim_path, conversation_report, parse_room_answer};
use serde::de::DeserializeOwned;
use serde_json::{Map, Value, json};
use std::fs;
use std::io;
use std::path::Path;

use request::{ClaimRequest, MatchRequest, NamesRequest, PaneRequest};
use scene::{Scene, with_site};

#[path = "claim/request.rs"]
mod request;
#[path = "claim/scene.rs"]
mod scene;

const ADDRESSEE_FILE: &str = "addressee.json";

pub(super) fn handle(name: &str, request: &Value) -> Result<Value, String> {
    match name {
        "claim" => claim_operation(request),
        "conversation_report" => report_operation(request),
        "claim_path" => path_operation(request),
        "matches_session" => matches_operation(request),
        "names_a_session" => names_operation(request),
        "active_conversation" => active_operation(request),
        other => Err(format!("unknown claim operation {other}")),
    }
}

fn parse<T: DeserializeOwned>(request: &Value) -> Result<T, String> {
    T::deserialize(request).map_err(|error| error.to_string())
}

fn claim_operation(request: &Value) -> Result<Value, String> {
    let ClaimRequest {
        scene,
        pane,
        room,
        now,
    } = parse(request)?;
    let won = with_site(&scene, |site| claim(site, &pane, &room, now))?;
    let files = written_files(&scene.directory, &pane).map_err(|error| error.to_string())?;
    Ok(json!({"result": won, "files": files}))
}

fn written_files(directory: &Path, pane: &str) -> io::Result<Map<String, Value>> {
    [directory.join(ADDRESSEE_FILE), claim_path(directory, pane)]
        .into_iter()
        .filter(|path| path.exists())
        .map(|path| read_entry(directory, &path))
        .collect()
}

fn read_entry(directory: &Path, path: &Path) -> io::Result<(String, Value)> {
    let text = fs::read_to_string(path)?;
    Ok((relative_text(directory, path), Value::String(text)))
}

fn relative_text(directory: &Path, path: &Path) -> String {
    path.strip_prefix(directory)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

fn report_operation(request: &Value) -> Result<Value, String> {
    let scene: Scene = parse(request)?;
    with_site(&scene, conversation_report).map(Value::String)
}

fn active_operation(request: &Value) -> Result<Value, String> {
    let scene: Scene = parse(request)?;
    let text = with_site(&scene, conversation_report)?;
    serde_json::from_str(&text).map_err(|error| error.to_string())
}

fn path_operation(request: &Value) -> Result<Value, String> {
    let PaneRequest { directory, pane } = parse(request)?;
    let path = claim_path(&directory, &pane);
    Ok(Value::String(relative_text(&directory, &path)))
}

fn matches_operation(request: &Value) -> Result<Value, String> {
    let MatchRequest {
        conversation,
        pane,
        host,
    } = parse(request)?;
    Ok(json!(conversation.is_some_and(
        |found| found.host == host && found.pane == pane
    )))
}

fn names_operation(request: &Value) -> Result<Value, String> {
    let NamesRequest { conversation } = parse(request)?;
    let answer = parse_room_answer("conversation", &conversation.to_string());
    Ok(json!(answer.is_ok_and(|found| found.is_some())))
}
