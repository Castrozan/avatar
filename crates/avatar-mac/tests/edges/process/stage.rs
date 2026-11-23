use std::net::TcpListener;
use std::path::Path;

use avatar_mac::edge::stage::{BrowserProcesses, MacSurface};
use avatar_mac::stage::StageSurface;
use avatar_seat::local::SeatConfig;

use crate::fakes::{TestResult, executable, logged};

const WINDOW_COMMAND: &str = "-n -a Chromium --args \
     --user-data-dir=/home/x/.local/share/avatar/window-browser \
     --app=http://localhost:8770/?viewer --window-size=640,360 --no-first-run \
     --no-default-browser-check";

fn seat_config() -> SeatConfig {
    SeatConfig {
        avatar_host: "host".to_owned(),
        browser_application: "Chromium".to_owned(),
        home: "/home/x".to_owned(),
        holder_program: String::new(),
    }
}

fn surface(directory: &Path) -> std::io::Result<MacSurface> {
    let log = directory.join("log");
    let record = format!("printf '%s\\n' \"$*\" >> {}", log.display());
    Ok(MacSurface {
        avatar_command: executable(directory, "avatar-command", &record)?,
        open: executable(directory, "open", &record)?,
        config: seat_config(),
        processes: BrowserProcesses::new(executable(directory, "pkill", &record)?),
    })
}

#[test]
fn avatar_commands_run_with_their_arguments() -> TestResult {
    let directory = tempfile::tempdir()?;
    let mut surface = surface(directory.path())?;
    surface.avatar_command(&["speak".to_owned(), "hi".to_owned()], 5.0)?;
    assert_eq!(logged(&directory.path().join("log")).trim(), "speak hi");
    Ok(())
}

#[test]
fn the_window_opens_with_its_own_profile_and_size() -> TestResult {
    let directory = tempfile::tempdir()?;
    let mut surface = surface(directory.path())?;
    surface.open_window("?viewer", 8770)?;
    let log = logged(&directory.path().join("log"));
    assert_eq!(log.trim(), WINDOW_COMMAND);
    Ok(())
}

#[test]
fn hiding_the_window_ends_the_processes_of_its_profile() -> TestResult {
    let directory = tempfile::tempdir()?;
    surface(directory.path())?.hide_window_surface()?;
    let log = logged(&directory.path().join("log"));
    assert_eq!(
        log.trim(),
        "-TERM -f -- --user-data-dir=/home/x/.local/share/avatar/window-browser"
    );
    Ok(())
}

#[test]
fn a_port_answers_while_something_listens_on_it() -> TestResult {
    let directory = tempfile::tempdir()?;
    let mut surface = surface(directory.path())?;
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    assert!(surface.answers(port));
    drop(listener);
    assert!(!surface.answers(port));
    Ok(())
}
