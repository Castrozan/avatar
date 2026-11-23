use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use avatar_audio_graph::pulse::adapter::PactlAdapter;
use avatar_audio_graph::pulse::control::PulseControl;
use tempfile::TempDir;

const LOAD_ARGUMENTS: [&str; 3] = [
    "load-module",
    "module-null-sink",
    "sink_properties=device.description=Avatar Virtual Microphone",
];

fn text(error: impl std::fmt::Display) -> String {
    error.to_string()
}

fn fake_pactl(directory: &Path, stdout: &str, status: i32) -> Result<PathBuf, String> {
    let script = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\nprintf '%s' '{stdout}'\nexit {status}\n",
        directory.join("argv.txt").display()
    );
    let program = directory.join("pactl");
    fs::write(&program, script).map_err(text)?;
    fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).map_err(text)?;
    Ok(program)
}

fn recorded_argv(directory: &Path) -> Result<Vec<String>, String> {
    let recorded = fs::read_to_string(directory.join("argv.txt")).map_err(text)?;
    Ok(recorded.lines().map(str::to_owned).collect())
}

#[test]
fn run_passes_arguments_verbatim_and_trims_stdout() -> Result<(), String> {
    let directory = TempDir::new().map_err(text)?;
    let program = fake_pactl(directory.path(), "  536870916  \n", 0)?;
    let output = PactlAdapter::new(&program)
        .run(&LOAD_ARGUMENTS)
        .map_err(text)?;
    assert_eq!(output, "536870916");
    assert_eq!(recorded_argv(directory.path())?, LOAD_ARGUMENTS);
    Ok(())
}

#[test]
fn a_non_zero_exit_is_a_pulse_failure() -> Result<(), String> {
    let directory = TempDir::new().map_err(text)?;
    let program = fake_pactl(directory.path(), "", 3)?;
    let outcome = PactlAdapter::new(&program)
        .run(&["unload-module", "9"])
        .map_err(text);
    assert_eq!(outcome, Err("pactl exited with exit status: 3".to_owned()));
    assert_eq!(recorded_argv(directory.path())?, ["unload-module", "9"]);
    Ok(())
}

#[test]
fn a_program_that_cannot_start_is_a_pulse_failure() {
    let directory = TempDir::new().map_err(text);
    let missing = directory.map(|directory| directory.path().join("missing-pactl"));
    let outcome =
        missing.and_then(|program| PactlAdapter::new(program).run(&["list"]).map_err(text));
    assert!(outcome.is_err_and(|message| message.starts_with("cannot start ")));
}
