use std::error::Error;
use std::fs;
use std::process::{Command, Output};

use super::{Machine, TestResult, text};

const STATUS: &str = "{\"open\": true, \"live\": false, \"seat\": \"room\", \"last_heard\": 0}";
const FORWARDED: &str =
    "-o BatchMode=yes -o ConnectTimeout=10 elsewhere env HERDR_PANE_ID=p1 avatar-talk say hi";
const OPENED: &str = "-o BatchMode=yes -o ConnectTimeout=10 chise avatar-talk open --seat remote";

fn configured(machine: &Machine, arguments: &[&str]) -> Command {
    let mut command = machine.command(&["mac", "talk"]);
    command.args(arguments);
    command
        .env("AVATAR_HOST", "chise")
        .env("AVATAR_BROWSER_APPLICATION", "/Applications/Chromium.app")
        .env("AVATAR_CAMERA_START_SCRIPT", machine.path("start.lua"))
        .env("AVATAR_SEAT_HOLDER_PYTHON", "python3")
        .env("AVATAR_SEAT_HOLDER", "seat_holder.py");
    command
}

fn mac_talk(machine: &Machine, arguments: &[&str]) -> Result<Output, Box<dyn Error>> {
    Ok(configured(machine, arguments).output()?)
}

fn ssh_machine(ending: &str) -> Result<Machine, Box<dyn Error>> {
    let machine = Machine::with_tools(&[])?;
    let log = machine.path("ssh.log");
    machine.install(
        "ssh",
        &format!("echo \"$@\" > {}\necho '{STATUS}'\n{ending}", log.display()),
    )?;
    Ok(machine)
}

fn ssh_log(machine: &Machine) -> Result<String, Box<dyn Error>> {
    Ok(fs::read_to_string(machine.path("ssh.log"))?
        .trim()
        .to_owned())
}

#[test]
fn a_command_for_another_room_is_forwarded_and_its_status_is_returned() -> TestResult {
    let machine = ssh_machine("exit 7")?;
    let mut command = configured(&machine, &["say", "hi"]);
    let output = command
        .env("AVATAR_ROOM", "elsewhere")
        .env("HERDR_PANE_ID", "p1")
        .output()?;
    assert_eq!(output.status.code(), Some(7));
    assert_eq!(ssh_log(&machine)?, FORWARDED);
    Ok(())
}

#[test]
fn opening_the_remote_seat_asks_the_avatar_host_and_prints_its_status() -> TestResult {
    let machine = ssh_machine("exit 0")?;
    let output = mac_talk(&machine, &["open", "--seat", "remote"])?;
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(text(&output.stdout).trim(), STATUS);
    assert_eq!(ssh_log(&machine)?, OPENED);
    Ok(())
}

#[test]
fn the_live_voice_opened_on_the_host_needs_an_agent_pane() -> TestResult {
    let machine = Machine::with_tools(&[])?;
    let output = mac_talk(&machine, &["open", "--live", "--seat", "remote"])?;
    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).contains("must run from an agent's herdr pane"));
    Ok(())
}

#[test]
fn a_mac_talk_without_the_seat_holder_program_names_the_missing_variable() -> TestResult {
    let output = Machine::with_tools(&[])?.run(&["mac", "talk", "status"])?;
    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).contains("AVATAR_SEAT_HOLDER_PYTHON is not set"));
    Ok(())
}
