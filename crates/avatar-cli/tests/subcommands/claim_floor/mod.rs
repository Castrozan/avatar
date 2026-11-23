mod agent;
mod pane;
mod speaker;

use std::error::Error;
use std::fs;
use std::process::Output;

use serde_json::{Value, json};

use crate::machine::{Machine, TestResult};

pub(super) const LOG_COMMAND: &str = r#"echo "$@" >> "${0%/*}/../log""#;
pub(super) const VOYAGER_LISTING: &str =
    r#"echo '{"s": {"paneId": "wW:pPZ", "servant": "Voyager"}}'"#;
pub(super) const PANES: &str = r#"echo '{"result": {"panes": [{"pane_id": "w1-1", "cwd": "/x"}, {"pane_id": "w1-2", "cwd": "/y"}]}}'"#;
pub(super) const SERVANTS: &str = r#"echo '{"a": {"paneId": "w1-1", "servant": "Circe"}, "b": {"paneId": "w1-2", "servant": "Medea"}}'"#;

pub(super) fn machine_with(tools: &[(&str, &str)]) -> Result<Machine, Box<dyn Error>> {
    let machine = Machine::with_tools(tools)?;
    fs::write(
        machine.path("characters.txt"),
        "Voyager | explorer\nCirce | witch\nMedea | sorceress\n",
    )?;
    Ok(machine)
}

pub(super) fn claim(
    machine: &Machine,
    arguments: &[&str],
    pane: Option<&str>,
) -> std::io::Result<Output> {
    let mut command = machine.command(arguments);
    command.env("AVATAR_SERVANT_CHARACTERS", machine.path("characters.txt"));
    if let Some(id) = pane {
        command.env("HERDR_PANE_ID", id);
    }
    command.output()
}

pub(super) fn hold(machine: &Machine, pane: &str, servant: &str, host: Option<&str>) -> TestResult {
    let record = json!({"pane": pane, "servant": servant, "heard_through": 2, "host": host});
    fs::write(machine.runtime().join("addressee.json"), record.to_string())?;
    Ok(())
}

pub(super) fn holder(machine: &Machine) -> Result<Value, Box<dyn Error>> {
    Ok(serde_json::from_str(&fs::read_to_string(
        machine.runtime().join("addressee.json"),
    )?)?)
}
