mod audio;
mod transcribe;

use std::error::Error;
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Child, Command, Output, Stdio};

pub(crate) type TestResult = Result<(), Box<dyn Error>>;

pub(crate) fn script(directory: &Path, name: &str, body: &str) -> TestResult {
    let path = directory.join(name);
    fs::write(&path, format!("#!/bin/sh\n{body}\n"))?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755))?;
    Ok(())
}

fn spawn_isolated(arguments: &[&str], home: &Path) -> Result<Child, Box<dyn Error>> {
    Ok(Command::new(env!("CARGO_BIN_EXE_avatar"))
        .args(arguments)
        .env_clear()
        .env("PATH", home.join("bin"))
        .env("HOME", home)
        .env("XDG_RUNTIME_DIR", home.join("run"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?)
}

pub(crate) fn run_isolated(
    arguments: &[&str],
    home: &Path,
    input: &[u8],
) -> Result<Output, Box<dyn Error>> {
    let mut child = spawn_isolated(arguments, home)?;
    child.stdin.take().ok_or("stdin")?.write_all(input)?;
    Ok(child.wait_with_output()?)
}
