use std::time::Duration;

use super::fake_program::fake_program;
use crate::support::Outcome;
use avatar_listening::listener::bounded_command::run_bounded;
use avatar_listening::listener::units::{LaunchdUnits, SessionUnits, SystemdUnits};
use avatar_listening::user_units::launchd::LaunchctlPrint;
use avatar_listening::user_units::launchd::effects::LaunchctlControl;
use avatar_listening::user_units::launchd::process::ProcessLaunchctl;

const RECORDING_LAUNCHCTL: &str = "echo \"$@\" > \"$0.args\"; printf '\\tstate = running\\n'";

#[test]
fn launchctl_print_returns_standard_output() -> Outcome {
    let directory = tempfile::tempdir()?;
    let program = fake_program(directory.path(), "launchctl", RECORDING_LAUNCHCTL)?;
    let mut launchctl = ProcessLaunchctl::new(&program);
    assert_eq!(launchctl.printed("gui/1/x"), "\tstate = running\n");
    let recorded = std::fs::read_to_string(directory.path().join("launchctl.args"))?;
    assert_eq!(recorded, "print gui/1/x\n");
    Ok(())
}

#[test]
fn kickstart_failure_is_reported_and_success_is_silent() -> Outcome {
    let directory = tempfile::tempdir()?;
    let failing = fake_program(directory.path(), "failing", "exit 3")?;
    let passing = fake_program(directory.path(), "passing", "exit 0")?;
    assert!(ProcessLaunchctl::new(failing).kickstart("gui/1/x").is_err());
    assert!(ProcessLaunchctl::new(passing).kickstart("gui/1/x").is_ok());
    Ok(())
}

#[test]
fn terminate_ignores_a_failing_launchctl() -> Outcome {
    let directory = tempfile::tempdir()?;
    let failing = fake_program(directory.path(), "failing", "exit 3")?;
    ProcessLaunchctl::new(failing).terminate("gui/1/x");
    Ok(())
}

#[test]
fn a_missing_launchctl_reads_as_inactive() {
    let mut launchd = LaunchdUnits::new(ProcessLaunchctl::new("/nonexistent/launchctl"), 503);
    assert!(!launchd.is_active("avatar-listener.service"));
    assert!(launchd.start("avatar-listener.service").is_err());
}

fn logged_after_first(directory: &std::path::Path) -> std::io::Result<String> {
    let log = std::fs::read_to_string(directory.join("systemctl.log"))?;
    Ok(log.lines().skip(1).collect::<Vec<_>>().join("|"))
}

#[test]
fn systemd_units_ask_systemctl_in_user_mode() -> Outcome {
    let directory = tempfile::tempdir()?;
    let logging = "echo \"$@\" >> \"$0.log\"";
    let program = fake_program(directory.path(), "systemctl", logging)?;
    let mut systemd = SystemdUnits::new(&program);
    assert!(systemd.is_active("avatar-listener.service"));
    systemd.start("avatar-listener.service")?;
    systemd.stop("avatar-live.service")?;
    let expected = "--user start avatar-listener.service|--user stop avatar-live.service";
    assert_eq!(logged_after_first(directory.path())?, expected);
    Ok(())
}

#[test]
fn a_failing_systemctl_means_inactive_and_failed_starts() -> Outcome {
    let directory = tempfile::tempdir()?;
    let program = fake_program(directory.path(), "systemctl", "exit 3")?;
    let mut systemd = SystemdUnits::new(&program);
    assert!(!systemd.is_active("avatar-listener.service"));
    assert!(systemd.start("avatar-listener.service").is_err());
    Ok(())
}

#[test]
fn a_command_that_outlives_its_timeout_is_killed() -> Outcome {
    let directory = tempfile::tempdir()?;
    let program = fake_program(directory.path(), "slow", "sleep 30")?;
    let outcome = run_bounded(&program, &[], Duration::from_millis(100));
    assert_eq!(
        outcome.err().map(|error| error.kind()),
        Some(std::io::ErrorKind::TimedOut)
    );
    Ok(())
}
