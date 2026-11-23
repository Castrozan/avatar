mod health;
mod process;

use std::error::Error;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use process::{ControlProcess, identify_agent};

type TestResult = Result<(), Box<dyn Error>>;

#[test]
fn control_subcommand_answers_identify_over_websocket() -> TestResult {
    let process = ControlProcess::start()?;
    let ack = identify_agent(&process.address)?;
    assert_eq!(ack["type"], "identifyAck");
    assert_eq!(ack["role"], "agent");
    Ok(())
}

#[test]
fn control_subcommand_serves_from_the_embedded_cast_without_any_variable() -> TestResult {
    let home = tempfile::tempdir()?;
    let environment: [(&str, &OsStr); 2] = [
        ("HOME", home.path().as_os_str()),
        ("AVATAR_DATA_DIRECTORY", home.path().as_os_str()),
    ];
    let process = ControlProcess::start_with(&[], &environment)?;
    assert_eq!(identify_agent(&process.address)?["type"], "identifyAck");
    assert!(home.path().join("cast/character-voices.json").is_file());
    Ok(())
}

fn write_characters(home: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let characters = home.join("characters.json");
    std::fs::write(&characters, r#"[{"name": "Test"}]"#)?;
    Ok(characters)
}

#[test]
fn control_subcommand_serves_with_the_real_speech_port_by_default() -> TestResult {
    let home = tempfile::tempdir()?;
    let characters = write_characters(home.path())?;
    let environment: [(&str, &OsStr); 2] = [
        ("HOME", home.path().as_os_str()),
        ("AVATAR_CHARACTER_VOICES", characters.as_os_str()),
    ];
    let process = ControlProcess::start_with(&[], &environment)?;
    assert_eq!(identify_agent(&process.address)?["type"], "identifyAck");
    Ok(())
}
