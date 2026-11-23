use std::fs;

use super::{Machine, TestResult, text};

#[test]
fn virtual_only_refuses_a_remote_seat_before_touching_anything() -> TestResult {
    let machine = Machine::with_tools(&[])?;
    let output = machine.run(&["start", "--virtual-only", "--seat", "remote"])?;
    assert_eq!(output.status.code(), Some(1));
    assert!(
        text(&output.stderr).contains("Virtual-only preparation requires the local meeting seat")
    );
    Ok(())
}

#[test]
fn an_unknown_seat_is_a_usage_error_with_status_two() -> TestResult {
    let machine = Machine::with_tools(&[])?;
    let output = machine.run(&["start", "--seat", "discord"])?;
    assert_eq!(output.status.code(), Some(2));
    Ok(())
}

#[test]
fn a_failing_target_start_repairs_audio_then_stops_the_target_and_exits_one() -> TestResult {
    let audio = "echo \"audio $@\" >> \"${0%/*}/../calls.log\"; case \"$1\" in status) echo '{\"running\": false}';; esac";
    let systemctl =
        "echo \"systemctl $@\" >> \"${0%/*}/../calls.log\"; case \"$2\" in start) exit 5;; esac";
    let machine = Machine::with_tools(&[("avatar-audio", audio), ("systemctl", systemctl)])?;
    let output = machine.run(&["start"])?;
    assert_eq!(output.status.code(), Some(1));
    let calls = fs::read_to_string(machine.path("calls.log"))?;
    assert!(calls.contains("audio start"));
    assert!(calls.contains("systemctl --user start avatar.target"));
    assert!(calls.contains("systemctl --user stop avatar.target"));
    Ok(())
}
