use std::error::Error;
use std::ffi::OsStr;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use serde_json::{Value, json};
use tokio_tungstenite::tungstenite::{Message, connect};

type TestResult = Result<(), Box<dyn Error>>;

struct ControlProcess {
    child: Child,
    address: String,
}

impl ControlProcess {
    fn start() -> Result<Self, Box<dyn Error>> {
        Self::start_with(&["--discard-speech"], &[])
    }

    fn start_with(
        arguments: &[&str],
        environment: &[(&str, &OsStr)],
    ) -> Result<Self, Box<dyn Error>> {
        let mut child = spawn_control(arguments, environment)?;
        let address = read_bound_address(&mut child)?;
        Ok(Self { child, address })
    }
}

fn spawn_control(
    arguments: &[&str],
    environment: &[(&str, &OsStr)],
) -> Result<Child, Box<dyn Error>> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_avatar"));
    command
        .args(["control", "--address", "127.0.0.1:0"])
        .args(arguments)
        .envs(environment.iter().copied())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    Ok(command.spawn()?)
}

fn read_bound_address(child: &mut Child) -> Result<String, Box<dyn Error>> {
    let stdout = child.stdout.take().ok_or("stdout was not piped")?;
    let mut line = String::new();
    BufReader::new(stdout).read_line(&mut line)?;
    Ok(line.trim().to_owned())
}

impl Drop for ControlProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn identify_agent(address: &str) -> Result<Value, Box<dyn Error>> {
    let (mut socket, _) = connect(format!("ws://{address}/"))?;
    socket.send(Message::Text(
        json!({"type": "identify", "role": "agent"})
            .to_string()
            .into(),
    ))?;
    let Message::Text(text) = socket.read()? else {
        return Err("expected a text frame".into());
    };
    Ok(serde_json::from_str(text.as_str())?)
}

#[test]
fn control_subcommand_answers_identify_over_websocket() -> TestResult {
    let process = ControlProcess::start()?;
    let ack = identify_agent(&process.address)?;
    assert_eq!(ack["type"], "identifyAck");
    assert_eq!(ack["role"], "agent");
    Ok(())
}

#[test]
fn control_subcommand_refuses_to_start_without_speech_configuration() -> TestResult {
    let output = Command::new(env!("CARGO_BIN_EXE_avatar"))
        .args(["control", "--address", "127.0.0.1:0"])
        .env_remove("AVATAR_CHARACTER_VOICES")
        .output()?;
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("avatar: "));
    Ok(())
}

fn write_characters(home: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let characters = home.join("characters.json");
    std::fs::write(&characters, r#"[{"name": "Test"}]"#)?;
    Ok(characters)
}

#[test]
fn control_subcommand_serves_with_the_real_speech_port_by_default() -> TestResult {
    let home = tempfile::tempdir()?;
    let characters = write_characters(home.path())?;
    let environment: [(&str, &OsStr); 2] = [
        ("HOME", home.path().as_os_str()),
        ("AVATAR_CHARACTER_VOICES", characters.as_os_str()),
    ];
    let process = ControlProcess::start_with(&[], &environment)?;
    assert_eq!(identify_agent(&process.address)?["type"], "identifyAck");
    Ok(())
}
