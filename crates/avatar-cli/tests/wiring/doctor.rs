use serde_json::{Value, json};

use super::{Machine, TestResult, text};

const ACTIVE: &str = "echo active";
const TRANSCRIBER_OK: &str = "exit 0";

#[test]
fn a_healthy_host_prints_ok_lines_and_exits_zero() -> TestResult {
    let machine =
        Machine::with_tools(&[("systemctl", ACTIVE), ("avatar-transcribe", TRANSCRIBER_OK)])?;
    let output = machine.run(&["doctor"])?;
    assert_eq!(
        text(&output.stdout),
        "ok   hex.service: active\nok   avatar-wake.service: active\nok   transcription: answers\n"
    );
    assert_eq!(output.status.code(), Some(0));
    Ok(())
}

const HEX_INACTIVE: &str = "case \"$3\" in hex.service) echo inactive;; *) echo active;; esac";

fn inactive_hex_report() -> Result<(Value, Option<i32>), Box<dyn std::error::Error>> {
    let tools = [
        ("systemctl", HEX_INACTIVE),
        ("avatar-transcribe", TRANSCRIBER_OK),
    ];
    let output = Machine::with_tools(&tools)?.run(&["doctor", "--json"])?;
    Ok((
        serde_json::from_slice(&output.stdout)?,
        output.status.code(),
    ))
}

#[test]
fn an_inactive_unit_fails_the_json_report_and_exits_one() -> TestResult {
    let (findings, code) = inactive_hex_report()?;
    let expected = json!({"name": "hex.service", "healthy": false, "detail": "inactive"});
    assert_eq!(findings[0], expected);
    assert_eq!(code, Some(1));
    Ok(())
}

#[test]
fn a_transcriber_that_fails_reports_its_last_stderr_line() -> TestResult {
    let transcriber = "echo first >&2; echo no model >&2; exit 3";
    let machine =
        Machine::with_tools(&[("systemctl", ACTIVE), ("avatar-transcribe", transcriber)])?;
    let output = machine.run(&["doctor"])?;
    assert!(text(&output.stdout).ends_with("FAIL transcription: no model\n"));
    assert_eq!(output.status.code(), Some(1));
    Ok(())
}

#[test]
fn an_unreachable_seat_host_is_reported_through_the_fake_ssh() -> TestResult {
    let machine = Machine::with_tools(&[
        ("ssh", "echo \"$@\" >> \"${0%/*}/../ssh.log\"; exit 255"),
        ("launchctl", "exit 1"),
        ("avatar-transcribe", TRANSCRIBER_OK),
    ])?;
    let output = machine.run(&["doctor", "--seat-of", "macbook"])?;
    assert!(text(&output.stdout).contains("FAIL"));
    assert_eq!(output.status.code(), Some(1));
    let log = std::fs::read_to_string(machine.path("ssh.log"))?;
    assert!(log.contains("macbook"));
    Ok(())
}
