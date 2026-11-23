use std::path::Path;

use avatar_listening::user_units::launchd::process::ProcessLaunchctl;
use avatar_mac::edge::stage::LaunchdStageUnits;
use avatar_mac::stage::StageUnits;

use crate::fakes::{TestResult, executable, logged};

const CONTROL: &str = "avatar-control-server.service";

const SCRIPT: &str = "dir=$(dirname \"$0\")\n\
     echo \"$@\" >> \"$dir/log\"\n\
     case \"$1\" in\n\
     print) [ -f \"$dir/running\" ] && printf 'state\\n\\tstate = running\\n' ;;\n\
     kickstart) touch \"$dir/running\" ;;\n\
     kill) rm -f \"$dir/running\" ;;\n\
     esac";

fn launchctl(directory: &Path) -> std::io::Result<std::path::PathBuf> {
    executable(directory, "launchctl", SCRIPT)
}

#[test]
fn a_unit_is_active_only_while_launchd_prints_it_running() -> TestResult {
    let directory = tempfile::tempdir()?;
    let mut units =
        LaunchdStageUnits::new(ProcessLaunchctl::new(launchctl(directory.path())?), 501);
    assert!(!units.unit_active(CONTROL));
    units.start_unit(CONTROL)?;
    assert!(units.unit_active(CONTROL));
    Ok(())
}

#[test]
fn starting_kickstarts_the_agent_of_the_user() -> TestResult {
    let directory = tempfile::tempdir()?;
    let mut units =
        LaunchdStageUnits::new(ProcessLaunchctl::new(launchctl(directory.path())?), 501);
    units.start_unit(CONTROL)?;
    let log = logged(&directory.path().join("log"));
    assert!(log.contains("kickstart gui/501/com.dotfiles.avatar-control"));
    Ok(())
}

#[test]
fn stopping_terminates_the_agent_and_waits_until_it_is_gone() -> TestResult {
    let directory = tempfile::tempdir()?;
    let mut units =
        LaunchdStageUnits::new(ProcessLaunchctl::new(launchctl(directory.path())?), 501);
    units.start_unit(CONTROL)?;
    units.stop_unit(CONTROL)?;
    assert!(!units.unit_active(CONTROL));
    Ok(())
}

#[test]
fn an_unknown_unit_cannot_be_started() -> TestResult {
    let directory = tempfile::tempdir()?;
    let mut units =
        LaunchdStageUnits::new(ProcessLaunchctl::new(launchctl(directory.path())?), 501);
    assert!(units.start_unit("unknown.service").is_err());
    Ok(())
}
