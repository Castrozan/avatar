use std::error::Error;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

pub(crate) const LOG: &str = ".local/share/avatar/seat-holder.log";
const BROWSER: &str = "/Applications/Chromium.app";
const PATIENCE: Duration = Duration::from_secs(20);
const POLL: Duration = Duration::from_millis(50);

pub(crate) struct Seat {
    home: tempfile::TempDir,
}

impl Seat {
    pub(crate) fn with_ssh(body: &str) -> Result<Self, Box<dyn Error>> {
        let seat = Self {
            home: tempfile::tempdir()?,
        };
        fs::create_dir_all(seat.path("bin"))?;
        let ssh = seat.path("bin/ssh");
        fs::write(&ssh, format!("#!/bin/sh\n{body}\n"))?;
        fs::set_permissions(&ssh, fs::Permissions::from_mode(0o755))?;
        Ok(seat)
    }

    pub(crate) fn path(&self, relative: &str) -> PathBuf {
        self.home.path().join(relative)
    }

    pub(crate) fn holder(&self, query: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_avatar"));
        command.args(["mac", "seat-holder"]).args(query);
        command.env_clear().stdin(Stdio::null());
        command.envs(self.environment());
        command
    }

    fn search_path(&self) -> String {
        let inherited = std::env::var("PATH").unwrap_or_default();
        format!("{}:{inherited}", self.path("bin").display())
    }

    fn environment(&self) -> [(&'static str, String); 4] {
        [
            ("PATH", self.search_path()),
            ("HOME", self.home.path().display().to_string()),
            ("AVATAR_HOST", "chise".to_owned()),
            ("AVATAR_BROWSER_APPLICATION", BROWSER.to_owned()),
        ]
    }

    pub(crate) fn log(&self) -> String {
        fs::read_to_string(self.path(LOG)).unwrap_or_default()
    }
}

fn has_finished(child: &mut Child) -> bool {
    child.try_wait().is_ok_and(|status| status.is_some())
}

pub(crate) fn finished_within_patience(child: &mut Child) -> bool {
    let deadline = Instant::now() + PATIENCE;
    while !has_finished(child) && Instant::now() < deadline {
        sleep(POLL);
    }
    let finished = has_finished(child);
    let _ = child.kill();
    finished
}

pub(crate) fn wait_for(path: &Path) {
    let deadline = Instant::now() + PATIENCE;
    while !path.exists() && Instant::now() < deadline {
        sleep(POLL);
    }
    assert!(path.exists(), "{} never appeared", path.display());
}
