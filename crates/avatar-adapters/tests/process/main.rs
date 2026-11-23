use std::error::Error;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Instant;

use avatar_adapters::microphone::{ParecMicrophone, capture_arguments};
use avatar_adapters::process::SubprocessRunner;
use avatar_wake::listener::Microphone;
use avatar_wake::reach::CommandRunner;

fn script(directory: &Path, name: &str, body: &str) -> Result<PathBuf, Box<dyn Error>> {
    let path = directory.join(name);
    fs::write(&path, format!("#!/bin/sh\n{body}\n"))?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755))?;
    Ok(path)
}

fn command(path: &Path, arguments: &[&str]) -> Vec<String> {
    let mut words = vec![path.to_string_lossy().into_owned()];
    words.extend(arguments.iter().map(|argument| (*argument).to_owned()));
    words
}

#[test]
fn runner_returns_stdout_and_passes_argv_and_stdin_closed() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let tool = script(directory.path(), "tool", "printf '%s|' \"$@\"")?;
    let output = SubprocessRunner.stdout_of(&command(&tool, &["a b", "c"]), 5);
    assert_eq!(output.as_deref(), Some("a b|c|"));
    Ok(())
}

fn assert_nothing(words: &[String]) {
    assert_eq!(SubprocessRunner.stdout_of(words, 5), None);
}

#[test]
fn runner_reports_nothing_on_failure_missing_program_and_bad_utf8() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let failing = script(directory.path(), "failing", "echo out; exit 3")?;
    let binary = script(directory.path(), "binary", "printf '\\377'")?;
    assert_nothing(&command(&failing, &[]));
    assert_nothing(&command(&binary, &[]));
    assert_nothing(&["/nonexistent/x".to_owned()]);
    assert_nothing(&[]);
    Ok(())
}

#[test]
fn runner_kills_a_command_that_exceeds_the_timeout() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let slow = script(directory.path(), "slow", "exec sleep 30")?;
    let started = Instant::now();
    assert_eq!(SubprocessRunner.stdout_of(&command(&slow, &[]), 1), None);
    assert!(started.elapsed().as_secs() < 10);
    Ok(())
}

fn frame_sizes(microphone: &mut ParecMicrophone) -> Vec<usize> {
    std::iter::from_fn(|| microphone.next_frame())
        .map(|frame| frame.len())
        .collect()
}

#[test]
fn microphone_frames_are_1024_bytes_with_a_short_final_frame() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let argv_file = directory.path().join("argv");
    let mut microphone = fake_parec(directory.path(), &argv_file)?;
    microphone.start();
    assert_eq!(frame_sizes(&mut microphone), [1024, 1024, 452]);
    microphone.stop();
    assert_eq!(argv_lines(&argv_file)?, capture_arguments());
    Ok(())
}

fn argv_lines(path: &Path) -> Result<Vec<String>, Box<dyn Error>> {
    let argv = fs::read_to_string(path)?;
    Ok(argv.lines().map(str::to_owned).collect())
}

fn fake_parec(directory: &Path, argv_file: &Path) -> Result<ParecMicrophone, Box<dyn Error>> {
    let body = format!(
        "printf '%s\\n' \"$@\" > {}; head -c 2500 /dev/zero",
        argv_file.display()
    );
    Ok(ParecMicrophone::new(script(directory, "parec", &body)?))
}

#[test]
fn microphone_without_a_started_capture_yields_nothing() {
    let mut microphone = ParecMicrophone::new("/nonexistent/parec");
    microphone.start();
    assert_eq!(microphone.next_frame(), None);
    microphone.stop();
}
