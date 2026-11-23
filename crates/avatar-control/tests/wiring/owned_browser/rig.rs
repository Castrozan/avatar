use std::error::Error;
use std::fmt::Display;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::Duration;

use avatar_control::owned_browser::{OwnedBrowser, Timing};

pub(crate) type TestResult = Result<(), Box<dyn Error>>;

pub(crate) fn check(condition: bool, detail: impl Display) -> TestResult {
    if condition {
        return Ok(());
    }
    Err(detail.to_string().into())
}

const PREAMBLE: &str = "for argument in \"$@\"; do case \"$argument\" in --user-data-dir=*) directory=\"${argument#--user-data-dir=}\";; esac; done\nprintf '%s\\n' \"$@\" > \"$directory/arguments.txt\"\n";

pub(crate) const SERVING: &str = "printf '4242\\n/devtools/browser/x' > \"$directory/DevToolsActivePort\"\ntrap 'kill $sleeper 2>/dev/null; exit 0' TERM\nsleep 10 &\nsleeper=$!\nwait $sleeper";
pub(crate) const EXITING: &str = "exit 0";
pub(crate) const SILENT: &str =
    "trap 'kill $sleeper 2>/dev/null; exit 0' TERM\nsleep 10 &\nsleeper=$!\nwait $sleeper";
pub(crate) const STUBBORN_SILENT: &str = "trap '' TERM\nwhile true; do sleep 1; done";
pub(crate) const STUBBORN: &str = "printf '4242\\n/x' > \"$directory/DevToolsActivePort\"\ntrap '' TERM\nwhile true; do sleep 1; done";

pub(crate) fn fake_browser(directory: &Path, body: &str) -> Result<OwnedBrowser, Box<dyn Error>> {
    let program = directory.join("chromium");
    fs::write(&program, format!("#!/bin/sh\n{PREAMBLE}{body}\n"))?;
    fs::set_permissions(&program, fs::Permissions::from_mode(0o755))?;
    let mut browser = OwnedBrowser::for_profile(directory.join("data/avatar/browser"))?;
    browser.program = program.to_string_lossy().into_owned();
    browser.timing = quick_timing();
    Ok(browser)
}

fn quick_timing() -> Timing {
    Timing {
        poll_interval: Duration::from_millis(10),
        endpoint_attempts: 100,
        stop_attempts: 30,
        supervision_interval: Duration::from_millis(10),
    }
}

pub(crate) struct Cleanup(pub(crate) std::path::PathBuf);

impl Drop for Cleanup {
    fn drop(&mut self) {
        let identifier = fs::read_to_string(self.0.join("browser.pid")).unwrap_or_default();
        let _ = std::process::Command::new("kill")
            .args(["-9", identifier.trim()])
            .output();
    }
}
