use super::super::helpers::{Failure, strings};
use crate::support::{Outcome, fixture};
use avatar_listening::listener::capture::{
    DISCORD_DEVICE, FRAME_BYTES, capture_command, command_by_device,
};
use serde_json::Value;

#[test]
fn every_oracle_pulse_command_matches_python() -> Outcome {
    let document = fixture("edges/fixtures/capture.json")?;
    let commands = document["commands"].as_object().ok_or("commands")?;
    assert_eq!(commands.len(), 4);
    for (device, expected) in commands
        .iter()
        .filter(|(device, _)| *device != DISCORD_DEVICE)
    {
        assert_eq!(capture_command(device), strings(expected)?, "{device}");
    }
    Ok(())
}

#[test]
fn the_frame_is_the_python_frame_size() -> Outcome {
    let document = fixture("edges/fixtures/capture.json")?;
    assert_eq!(
        u64::try_from(FRAME_BYTES)?,
        document["frame_bytes"].as_u64().ok_or("bytes")?
    );
    Ok(())
}

fn mapped(device: &str) -> Vec<String> {
    command_by_device(|name| vec!["machine".to_owned(), name.to_owned()])(device)
}

fn check_platform(document: &Value, platform: &str) -> Result<(), Failure> {
    let expected = &document["by_device"][platform];
    assert_eq!(mapped(DISCORD_DEVICE), strings(&expected["discord"])?);
    assert_eq!(mapped("headset"), strings(&expected["headset"])?);
    Ok(())
}

#[test]
fn the_discord_device_captures_from_the_bridge_on_any_platform() -> Outcome {
    let document = fixture("edges/fixtures/capture.json")?;
    check_platform(&document, "linux")?;
    check_platform(&document, "darwin")
}

#[test]
fn every_discord_capture_gets_its_own_command_list() {
    let mapping = command_by_device(|_| Vec::new());
    mapping(DISCORD_DEVICE).push("changed".to_owned());
    assert_eq!(mapping(DISCORD_DEVICE), ["discord-voice", "capture"]);
}
