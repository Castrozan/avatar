use std::fs;

use serde_json::Value;

use super::{TestResult, run_isolated, script};

const PACTL: &str = "echo \"$@\" >> \"${0%/*}/../log\"; case \"$*\" in \"list short modules\") ;; \"--format=json list sinks\") echo '[{\"name\": \"AvatarMic\", \"mute\": true, \"volume\": 1}]';; *) exit 9;; esac";

fn home_with_tools(
    tools: &[(&str, &str)],
) -> Result<tempfile::TempDir, Box<dyn std::error::Error>> {
    let home = tempfile::tempdir()?;
    fs::create_dir_all(home.path().join("bin"))?;
    for (name, body) in tools {
        script(&home.path().join("bin"), name, body)?;
    }
    Ok(home)
}

#[test]
fn audio_status_prints_the_state_as_json_through_the_fake_pactl() -> TestResult {
    let home = home_with_tools(&[("pactl", PACTL)])?;
    let output = run_isolated(&["audio", "status"], home.path(), b"")?;
    let state: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(state["phase"], "absent");
    assert_eq!(state["channels"]["agent"]["mute"], true);
    assert!(output.status.success());
    Ok(())
}

#[test]
fn an_invalid_mute_value_fails_with_the_python_message_and_status_one() -> TestResult {
    let home = home_with_tools(&[("pactl", PACTL)])?;
    let output = run_isolated(&["audio", "mute", "agent", "2"], home.path(), b"")?;
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8(output.stderr)?.contains("Mute must be 0 or 1"));
    Ok(())
}

#[test]
fn a_negative_volume_is_parsed_then_refused_and_a_bad_channel_is_a_usage_error() -> TestResult {
    let home = home_with_tools(&[("pactl", PACTL)])?;
    let negative = run_isolated(&["audio", "volume", "human", "-5"], home.path(), b"")?;
    assert!(String::from_utf8(negative.stderr)?.contains("Volume must be between 0 and 100"));
    let usage = run_isolated(&["audio", "volume", "robot", "5"], home.path(), b"")?;
    assert_eq!(usage.status.code(), Some(2));
    Ok(())
}

#[test]
fn stop_runs_systemctl_then_the_in_process_audio_stop_and_exits_with_the_systemctl_status()
-> TestResult {
    let systemctl = "echo \"$@\" > \"${0%/*}/../systemctl.log\"; exit 4";
    let home = home_with_tools(&[("systemctl", systemctl), ("pactl", PACTL)])?;
    let output = run_isolated(&["stop"], home.path(), b"")?;
    assert_eq!(output.status.code(), Some(4));
    assert!(
        fs::read_to_string(home.path().join("systemctl.log"))?
            .starts_with("--user stop avatar-conversation.target")
    );
    let state: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(state["phase"], "absent");
    Ok(())
}
