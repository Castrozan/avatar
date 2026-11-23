use serde_json::Value;

use super::command::CommandRunner;
use super::failure::DoctorError;
use super::finding::Finding;
use super::page::PageProbe;

pub(crate) const WAKE_PAGE_PORT: u16 = 8771;

pub(crate) fn wake_microphone_hears(
    runner: &impl CommandRunner,
    probe: &impl PageProbe,
) -> Result<Finding, DoctorError> {
    if probe.serves(WAKE_PAGE_PORT) {
        return Ok(Finding::new("wake microphone", true, "capturing"));
    }
    let listening = conversation_open(runner)?;
    Ok(Finding::new(
        "wake microphone",
        listening,
        silent_page_detail(listening),
    ))
}

fn silent_page_detail(listening: bool) -> &'static str {
    if listening {
        "paused while a conversation listens"
    } else {
        "no capture page and no conversation"
    }
}

fn conversation_open(runner: &impl CommandRunner) -> Result<bool, DoctorError> {
    let output = runner.run(&["avatar-talk", "status"], &[])?;
    let status: Value = serde_json::from_slice(&output.stdout)?;
    Ok(status.get("open").and_then(Value::as_bool).unwrap_or(false))
}
