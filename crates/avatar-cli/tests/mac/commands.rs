use std::error::Error;
use std::path::Path;
use std::process::{Command, Output};

type TestResult = Result<(), Box<dyn Error>>;

fn avatar(arguments: &[&str]) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_avatar"));
    command.args(arguments).env_clear();
    command
}

fn configured(arguments: &[&str], home: &Path) -> Result<Output, Box<dyn Error>> {
    let mut command = avatar(arguments);
    command
        .env("HOME", home)
        .env("AVATAR_HOST", "chise")
        .env("AVATAR_BROWSER_APPLICATION", "/Applications/Chromium.app");
    Ok(command.output()?)
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

const COMMAND_NAMES: [&str; 8] = [
    "seat",
    "microphone",
    "play",
    "call-capture",
    "meeting",
    "listen",
    "seat-holder",
    "install-call-audio-drivers",
];

#[test]
fn the_mac_commands_are_listed() -> TestResult {
    let output = avatar(&["mac", "--help"]).output()?;
    let help = text(&output.stdout);
    for name in COMMAND_NAMES {
        assert!(help.contains(name), "{name} is missing from {help}");
    }
    Ok(())
}

#[test]
fn a_meeting_verb_outside_the_three_is_a_usage_error() -> TestResult {
    let home = tempfile::tempdir()?;
    let output = configured(&["mac", "meeting", "bogus"], home.path())?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        text(&output.stderr).trim(),
        "Usage: avatar-start | avatar-stop | avatar-controls"
    );
    Ok(())
}

#[test]
fn a_meeting_without_the_environment_names_the_missing_variable() -> TestResult {
    let output = avatar(&["mac", "meeting", "start"]).output()?;
    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).contains("AVATAR_HOST is not set"));
    Ok(())
}

#[test]
fn listening_options_follow_argparse_exit_codes() -> TestResult {
    let refused = avatar(&["mac", "listen", "start", "--bogus"]).output()?;
    assert_eq!(refused.status.code(), Some(2));
    let helped = avatar(&["mac", "listen", "start", "-h"]).output()?;
    assert_eq!(helped.status.code(), Some(0));
    let missing = avatar(&["mac", "listen", "start", "--language"]).output()?;
    assert_eq!(missing.status.code(), Some(2));
    Ok(())
}

#[test]
fn installing_no_drivers_changes_nothing() -> TestResult {
    let output = avatar(&["mac", "install-call-audio-drivers"]).output()?;
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty());
    Ok(())
}
