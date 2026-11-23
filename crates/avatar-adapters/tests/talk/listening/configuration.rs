use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use avatar_listening::conversation_state::ListenerPurpose;
use serde_json::{Value, json};

use super::{SYSTEMCTL, begin};
use crate::tools::{TestResult, Tools};

fn stored_configuration(directory: &Path) -> Result<Value, Box<dyn std::error::Error>> {
    let text = fs::read_to_string(directory.join("listener.json"))?;
    Ok(serde_json::from_str(&text)?)
}

#[test]
fn the_configuration_names_purpose_sources_and_settings() -> TestResult {
    let tools = Tools::with(&[("systemctl", SYSTEMCTL)])?;
    let state = tempfile::tempdir()?;
    assert_eq!(begin(&tools, state.path(), ListenerPurpose::Live), None);
    let sources = json!({"human": "AvatarRoomMicrophone"});
    let expected =
        json!({"purpose": "live", "sources": sources, "language": "en", "barge_in": true});
    assert_eq!(stored_configuration(state.path())?, expected);
    Ok(())
}

#[test]
fn the_configuration_is_private() -> TestResult {
    let tools = Tools::with(&[("systemctl", SYSTEMCTL)])?;
    let state = tempfile::tempdir()?;
    assert_eq!(begin(&tools, state.path(), ListenerPurpose::Live), None);
    let file = fs::metadata(state.path().join("listener.json"))?;
    assert_eq!(file.permissions().mode() & 0o777, 0o600);
    Ok(())
}

#[test]
fn the_live_purpose_starts_the_live_service() -> TestResult {
    let tools = Tools::with(&[("systemctl", SYSTEMCTL)])?;
    let state = tempfile::tempdir()?;
    assert_eq!(begin(&tools, state.path(), ListenerPurpose::Live), None);
    assert!(
        tools
            .read("systemctl.log")
            .ends_with("--user start avatar-live.service\n")
    );
    Ok(())
}

#[test]
fn a_failing_systemctl_start_carries_the_python_message() -> TestResult {
    let tools = Tools::with(&[(
        "systemctl",
        "case \"$2\" in start) exit 5;; *) exit 3;; esac",
    )])?;
    let state = tempfile::tempdir()?;
    let message = begin(&tools, state.path(), ListenerPurpose::Meeting).unwrap_or_default();
    assert!(message.contains("'avatar-listener.service'"), "{message}");
    assert!(message.ends_with("exit status 5."), "{message}");
    Ok(())
}
