use std::error::Error;
use std::ffi::OsStr;
use std::io::{BufRead, BufReader};
use std::process::{Child, ChildStdout, Command, Stdio};

use serde_json::{Value, json};
use tokio_tungstenite::tungstenite::{Message, connect};

pub(crate) struct ControlProcess {
    child: Child,
    pub(crate) address: String,
    pub(crate) http_address: String,
    _stdout: BufReader<ChildStdout>,
}

impl ControlProcess {
    pub(crate) fn start() -> Result<Self, Box<dyn Error>> {
        Self::start_with(&["--discard-speech"], &[])
    }

    pub(crate) fn start_with(
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

pub(crate) fn identify_agent(address: &str) -> Result<Value, Box<dyn Error>> {
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
