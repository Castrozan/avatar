use std::error::Error;
use std::io::{BufRead, BufReader};
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
        let mut child = spawn_control()?;
        let address = read_bound_address(&mut child)?;
        Ok(Self { child, address })
    }
}

fn spawn_control() -> Result<Child, Box<dyn Error>> {
    let child = Command::new(env!("CARGO_BIN_EXE_avatar"))
        .args(["control", "--address", "127.0.0.1:0"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(child)
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
