mod doctor;
mod holder;
mod listen;
mod start;
mod talk;
mod wake;

use std::error::Error;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::{Command, Output, Stdio};

const RUNTIME_DIRECTORY: &str = if cfg!(target_os = "macos") {
    "Library/Caches/avatar"
} else {
    "run/avatar"
};

pub(crate) type TestResult = Result<(), Box<dyn Error>>;

pub(crate) struct Machine {
    pub(crate) home: tempfile::TempDir,
}

impl Machine {
    pub(crate) fn with_tools(tools: &[(&str, &str)]) -> Result<Self, Box<dyn Error>> {
        let home = tempfile::tempdir()?;
        fs::create_dir_all(home.path().join("bin"))?;
        fs::create_dir_all(home.path().join(RUNTIME_DIRECTORY))?;
        let machine = Self { home };
        tools
            .iter()
            .try_for_each(|(name, body)| machine.install(name, body))?;
        Ok(machine)
    }

    pub(crate) fn install(&self, name: &str, body: &str) -> TestResult {
        let path = self.home.path().join("bin").join(name);
        fs::write(&path, format!("#!/bin/sh\n{body}\n"))?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755))?;
        Ok(())
    }

    pub(crate) fn path(&self, relative: &str) -> std::path::PathBuf {
        self.home.path().join(relative)
    }

    pub(crate) fn runtime(&self) -> std::path::PathBuf {
        self.path(RUNTIME_DIRECTORY)
    }

    pub(crate) fn wake_log(&self) -> Result<String, Box<dyn Error>> {
        Ok(fs::read_to_string(self.path("state/avatar/wake.log"))?)
    }

    pub(crate) fn run(&self, arguments: &[&str]) -> Result<Output, Box<dyn Error>> {
        Ok(self.command(arguments).output()?)
    }

    pub(crate) fn command(&self, arguments: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_avatar"));
        command.args(arguments).env_clear().stdin(Stdio::null());
        command.envs(self.environment());
        command
    }

    fn environment(&self) -> [(&'static str, std::path::PathBuf); 5] {
        [
            ("PATH", self.path("bin")),
            ("HOME", self.home.path().to_path_buf()),
            ("XDG_RUNTIME_DIR", self.path("run")),
            ("XDG_STATE_HOME", self.path("state")),
            ("AVATAR_AGENT_HOST", "testroom".into()),
        ]
    }
}

pub(crate) fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}
