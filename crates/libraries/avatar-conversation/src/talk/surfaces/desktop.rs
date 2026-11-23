use serde_json::Value;

use crate::talk::failure::TalkFailure;
use crate::talk::ports::exec::run_checked;
use crate::talk::ports::{Clock, Subprocess};

const WINDOW_CLASS: &str = "avatar-browser";
const CONTROLS_TITLE: &str = "localhost_/controls.html";
const HIDDEN_WORKSPACE: &str = "special:avatar";
const WINDOW_WAIT_SECONDS: f64 = 15.0;
const POLL_SECONDS: f64 = 0.2;
const FIRST_HYPRLAND_INSTANCE: &str = "0";

fn hyprctl_json(process: &mut impl Subprocess, kind: &str) -> Result<Vec<Value>, TalkFailure> {
    let argv = ["hyprctl", "--instance", FIRST_HYPRLAND_INSTANCE, "-j", kind];
    let output = run_checked(process, &argv, None)?;
    match serde_json::from_str(&output.stdout) {
        Ok(Value::Array(items)) => Ok(items),
        _ => Err(TalkFailure::new(
            "hyprctl answered something other than a JSON list",
        )),
    }
}

fn dispatch(process: &mut impl Subprocess, arguments: [&str; 2]) -> Result<(), TalkFailure> {
    let [name, argument] = arguments;
    let argv = [
        "hyprctl",
        "--instance",
        FIRST_HYPRLAND_INSTANCE,
        "dispatch",
        name,
        argument,
    ];
    run_checked(process, &argv, None).map(|_| ())
}

fn is_avatar_window(client: &Value) -> bool {
    client["class"] == WINDOW_CLASS && client["title"] != CONTROLS_TITLE
}

fn avatar_window(process: &mut impl Subprocess) -> Result<Option<Value>, TalkFailure> {
    let clients = hyprctl_json(process, "clients")?;
    Ok(clients.into_iter().find(is_avatar_window))
}

fn wait_for_avatar_window(boundary: &mut (impl Subprocess + Clock)) -> Result<Value, TalkFailure> {
    let deadline = boundary.monotonic() + WINDOW_WAIT_SECONDS;
    while boundary.monotonic() < deadline {
        match avatar_window(boundary)? {
            Some(window) => return Ok(window),
            None => boundary.sleep(POLL_SECONDS),
        }
    }
    Err(TalkFailure::new("The avatar window did not appear"))
}

fn scalar_text(value: &Value) -> String {
    value
        .as_str()
        .map_or_else(|| value.to_string(), str::to_owned)
}

fn focused_workspace(process: &mut impl Subprocess) -> Result<String, TalkFailure> {
    let monitors = hyprctl_json(process, "monitors")?;
    let focused = monitors
        .iter()
        .find(|item| item["focused"] == Value::Bool(true));
    let monitor = focused.ok_or_else(|| TalkFailure::new("No monitor is focused"))?;
    Ok(scalar_text(&monitor["activeWorkspace"]["id"]))
}

fn move_to_workspace(process: &mut impl Subprocess, destination: &str) -> Result<(), TalkFailure> {
    dispatch(process, ["movetoworkspacesilent", destination])
}

pub fn show(boundary: &mut (impl Subprocess + Clock)) -> Result<(), TalkFailure> {
    let window = wait_for_avatar_window(boundary)?;
    let target = format!("address:{}", scalar_text(&window["address"]));
    let workspace = focused_workspace(boundary)?;
    move_to_workspace(boundary, &format!("{workspace},{target}"))?;
    if window["floating"] == Value::Bool(true) {
        dispatch(boundary, ["togglefloating", &target])?;
    }
    Ok(())
}

pub fn hide(process: &mut impl Subprocess) -> Result<(), TalkFailure> {
    let Some(window) = avatar_window(process)? else {
        return Ok(());
    };
    let address = scalar_text(&window["address"]);
    move_to_workspace(process, &format!("{HIDDEN_WORKSPACE},address:{address}"))
}
