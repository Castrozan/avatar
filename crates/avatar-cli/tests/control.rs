use std::error::Error;
use std::ffi::OsStr;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdout, Command, Stdio};

use serde_json::{Value, json};
use tokio_tungstenite::tungstenite::{Message, connect};

type TestResult = Result<(), Box<dyn Error>>;

struct ControlProcess {
    child: Child,
    address: String,
    http_address: String,
    _stdout: BufReader<ChildStdout>,
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
        let stdout = child.stdout.take().ok_or("stdout was not piped")?;
        Self::announced(child, BufReader::new(stdout))
    }
}

impl ControlProcess {
    fn announced(child: Child, mut stdout: BufReader<ChildStdout>) -> Result<Self, Box<dyn Error>> {
        let address = read_line(&mut stdout)?;
        let http_address = read_line(&mut stdout)?;
        Ok(Self {
            child,
            address,
            http_address,
            _stdout: stdout,
        })
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
        .env("AVATAR_HTTP_PORT", "0")
        .envs(environment.iter().copied())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    Ok(command.spawn()?)
}

fn read_line(stdout: &mut BufReader<ChildStdout>) -> Result<String, Box<dyn Error>> {
    let mut line = String::new();
    stdout.read_line(&mut line)?;
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

fn fetch_health(address: &str) -> Result<Value, Box<dyn Error>> {
    use std::io::{Read, Write};
    let mut stream = std::net::TcpStream::connect(address)?;
    stream.write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")?;
    let mut raw = String::new();
    stream.read_to_string(&mut raw)?;
    let body = raw.split_once("\r\n\r\n").ok_or("no body")?.1;
    Ok(serde_json::from_str(body)?)
}

#[test]
fn control_subcommand_serves_health_on_the_http_port() -> TestResult {
    let process = ControlProcess::start()?;
    let health = fetch_health(&process.http_address)?;
    assert_eq!(health["status"], "ok");
    assert_eq!(health["state"], "idle");
    assert_eq!(health["agentConnected"], false);
    Ok(())
}

#[test]
fn control_subcommand_rejects_an_invalid_port_variable() -> TestResult {
    let output = Command::new(env!("CARGO_BIN_EXE_avatar"))
        .args(["control", "--discard-speech"])
        .env("AVATAR_WS_PORT", "not-a-port")
        .output()?;
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("AVATAR_WS_PORT"));
    Ok(())
}
