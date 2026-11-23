use std::error::Error;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Output};

type TestResult = Result<(), Box<dyn Error>>;

fn avatar_browser(data_home: &Path, arguments: &[&str]) -> Result<Output, Box<dyn Error>> {
    Ok(Command::new(env!("CARGO_BIN_EXE_avatar"))
        .arg("browser")
        .args(arguments)
        .env("XDG_DATA_HOME", data_home)
        .output()?)
}

fn fake_chromium(directory: &Path) -> Result<(), Box<dyn Error>> {
    let path = directory.join("chromium");
    fs::write(&path, "#!/bin/sh\nexit 0\n")?;
    Ok(fs::set_permissions(
        &path,
        fs::Permissions::from_mode(0o755),
    )?)
}

#[test]
fn stop_with_no_browser_succeeds_quietly() -> TestResult {
    let data = tempfile::tempdir()?;
    let output = avatar_browser(data.path(), &["stop"])?;
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    Ok(())
}

#[test]
fn controls_without_a_browser_explain_what_to_do() -> TestResult {
    let data = tempfile::tempdir()?;
    fake_chromium(data.path())?;
    let output = avatar_browser(data.path(), &["controls"])?;
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("Start the avatar before opening its controls")
    );
    Ok(())
}

#[test]
fn an_unknown_action_is_refused() -> TestResult {
    let data = tempfile::tempdir()?;
    let output = avatar_browser(data.path(), &["restart"])?;
    assert!(!output.status.success());
    Ok(())
}
