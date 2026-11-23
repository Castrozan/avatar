use std::error::Error;
use std::fs;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::machine::Machine;

pub(super) const SINKS: &str = r#"echo '[{"name": "AvatarMic", "mute": false, "volume": {"front-left": {"value_percent": "50%"}}}]'"#;
pub(super) const VOICE: &str = "head -c 960 /dev/zero; exec sleep 30";
pub(super) const HEARD: &str = r#"cat > "${0%/*}/../heard""#;

pub(super) struct Seat {
    pub(super) child: Child,
    pub(super) port: u16,
}

impl Drop for Seat {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub(super) fn free_port() -> Result<u16, Box<dyn Error>> {
    Ok(TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?
        .local_addr()?
        .port())
}

pub(super) fn seat_machine() -> Result<Machine, Box<dyn Error>> {
    let tools = [
        ("systemctl", "exit 0"),
        ("pactl", SINKS),
        ("pacat", HEARD),
        ("parec", VOICE),
    ];
    let machine = Machine::with_tools(&tools)?;
    fs::create_dir_all(machine.path("page"))?;
    Ok(machine)
}

pub(super) fn tools_path(machine: &Machine) -> String {
    let inherited = std::env::var("PATH").unwrap_or_default();
    format!("{}:{inherited}", machine.path("bin").display())
}

pub(super) fn start(machine: &Machine) -> Result<Seat, Box<dyn Error>> {
    spawn(machine, Some(&machine.path("page")))
}

pub(super) fn start_with_embedded_page(machine: &Machine) -> Result<Seat, Box<dyn Error>> {
    spawn(machine, None)
}

fn spawn(machine: &Machine, page: Option<&Path>) -> Result<Seat, Box<dyn Error>> {
    let port = free_port()?;
    let mut command = machine.command(&["remote-seat", "--port", &port.to_string()]);
    command.env("PATH", tools_path(machine));
    command.env("AVATAR_DATA_DIRECTORY", machine.path("data"));
    if let Some(directory) = page {
        command.env("AVATAR_SEAT_PAGE", directory);
    }
    let child = spawn_quiet(command)?;
    Ok(Seat { child, port })
}

fn spawn_quiet(mut command: Command) -> std::io::Result<Child> {
    command.stdout(Stdio::null()).stderr(Stdio::null()).spawn()
}

pub(super) fn serves_index(port: u16) -> bool {
    index_page(port).is_some_and(|reply| reply.contains("<!doctype html>"))
}

fn index_page(port: u16) -> Option<String> {
    let mut stream = TcpStream::connect((Ipv4Addr::LOCALHOST, port)).ok()?;
    stream
        .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .ok()?;
    let mut reply = String::new();
    stream.read_to_string(&mut reply).ok()?;
    Some(reply)
}

pub(super) fn health(port: u16) -> Option<Value> {
    let mut stream = TcpStream::connect((Ipv4Addr::LOCALHOST, port)).ok()?;
    stream
        .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .ok()?;
    let mut reply = String::new();
    stream.read_to_string(&mut reply).ok()?;
    serde_json::from_str(reply.split_once("\r\n\r\n")?.1).ok()
}

pub(super) fn wait_until(mut condition: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + Duration::from_secs(15);
    while Instant::now() < deadline && !condition() {
        sleep(Duration::from_millis(50));
    }
    condition()
}

pub(super) fn host_known(seat: &Seat) -> bool {
    health(seat.port).is_some_and(|body| body["meeting"] == true && body["listening"] == true)
}

pub(super) fn end_on_request(seat: &mut Seat) -> Result<bool, Box<dyn Error>> {
    let signalled = Command::new("kill")
        .arg(seat.child.id().to_string())
        .status()?;
    Ok(signalled.success() && seat.child.wait()?.success())
}
