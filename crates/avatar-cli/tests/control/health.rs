use std::error::Error;
use std::process::Command;

use serde_json::Value;

use super::process::ControlProcess;

type TestResult = Result<(), Box<dyn Error>>;

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
