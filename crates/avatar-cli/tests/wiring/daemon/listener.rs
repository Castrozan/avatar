use std::fs;
use std::path::Path;
use std::process::{Child, Stdio};
use std::time::Duration;

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;

use super::hex_server::serve_hex;
use super::waiting::{eventually, exit_code_within};
use crate::{Machine, TestResult, text};

const TURN_MODEL: &str = "/nix/store/ny4gbj36s6yvzrqj4c1v12rqqvnzip8b-smart-turn-v3.2-cpu.onnx";
const VOICE_MODEL: &str = "/nix/store/8njwlx068xgrzjhmv5h2z0hrs224w2r5-silero_vad.onnx";
const FRAME_SAMPLES: usize = 512;
pub(super) const CAPTURE_IDENTITY: &str = "capture.pid";
const CAPTURE: &str = "echo $$ > \"${0%/*}/../capture.pid\"; cat \"${0%/*}/../sample.pcm\"; \
    while [ -e \"$0\" ]; do head -c 1024 /dev/zero || exit; sleep 0.01; done";
const CONFIGURATION: &str = r#"{"sources":{"human":"fakemic"},"language":"en"}"#;

fn loud_sample(index: usize) -> [u8; 2] {
    let level: i16 = if index.is_multiple_of(2) { 8000 } else { -8000 };
    level.to_le_bytes()
}

fn speech_then_silence() -> Vec<u8> {
    let loud = (0..FRAME_SAMPLES * 20).flat_map(loud_sample);
    loud.chain(vec![0; FRAME_SAMPLES * 2 * 40]).collect()
}

fn write_hex_discovery(machine: &Machine, port: u16) -> TestResult {
    let directory = machine.path(".local/share/hex");
    fs::create_dir_all(&directory)?;
    let discovery = format!(r#"{{"port":{port},"token":"token"}}"#);
    fs::write(directory.join("local-api.json"), discovery)?;
    Ok(())
}

pub(super) fn models_installed() -> bool {
    Path::new(TURN_MODEL).exists() && Path::new(VOICE_MODEL).exists()
}

pub(super) fn start_daemon(machine: &Machine) -> Result<Child, Box<dyn std::error::Error>> {
    let mut command = machine.command_with_system_tools(&["listener"]);
    command.env("AVATAR_TURN_MODEL", TURN_MODEL);
    command.env("AVATAR_VOICE_ACTIVITY_MODEL", VOICE_MODEL);
    Ok(command.stderr(Stdio::null()).spawn()?)
}

fn terminate(child: Child) -> Result<Option<i32>, Box<dyn std::error::Error>> {
    kill(Pid::from_raw(i32::try_from(child.id())?), Signal::SIGTERM)?;
    exit_code_within(child, Duration::from_secs(20))
}

fn heard(transcript: &Path) -> bool {
    let content = fs::read_to_string(transcript).unwrap_or_default();
    content.contains("hello there") && content.contains("\"source\": \"human\"")
}

pub(super) fn prepared_machine() -> Result<Machine, Box<dyn std::error::Error>> {
    let machine = Machine::with_tools(&[("parec", CAPTURE)])?;
    fs::write(machine.path("sample.pcm"), speech_then_silence())?;
    write_hex_discovery(&machine, serve_hex("hello there")?)?;
    fs::write(machine.runtime().join("listener.json"), CONFIGURATION)?;
    Ok(machine)
}

#[test]
fn the_daemon_transcribes_a_captured_segment_through_hex_and_stops_on_sigterm() -> TestResult {
    if !models_installed() {
        return Ok(());
    }
    let machine = prepared_machine()?;
    let child = start_daemon(&machine)?;
    let transcript = machine.runtime().join("transcript.jsonl");
    let appeared = eventually(|| heard(&transcript), Duration::from_mins(1));
    assert_eq!(terminate(child)?, Some(0));
    assert!(appeared);
    Ok(())
}

#[test]
fn the_daemon_refuses_to_start_without_its_models() -> TestResult {
    let machine = Machine::with_tools(&[])?;
    fs::write(machine.runtime().join("listener.json"), r#"{"sources":{}}"#)?;
    let output = machine.run(&["listener"])?;
    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).contains("turn model not found"));
    Ok(())
}

#[test]
fn the_daemon_refuses_to_start_without_a_configuration() -> TestResult {
    let output = Machine::with_tools(&[])?.run(&["listener"])?;
    assert_eq!(output.status.code(), Some(1));
    assert!(text(&output.stderr).contains("listener.json"));
    Ok(())
}
