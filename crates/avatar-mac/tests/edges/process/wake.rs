use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::thread;

use avatar_mac::edge::stage::SystemWaiting;
use avatar_mac::edge::wake::MacWakeEdges;
use avatar_wake::mac::MacEdges;

use crate::fakes::{TestResult, executable, logged};

const LAUNCHCTL: &str = "[ \"$1\" = print ] && printf '\\tstate = running\\n'";

const SSH_WORDS: &str =
    "-o BatchMode=yes -o ConnectTimeout=5 chise avatar-listen read --since '0 1'";

fn fake_ssh(directory: &Path) -> std::io::Result<PathBuf> {
    let log = directory.join("ssh.log");
    let body = format!("echo \"$@\" > {}\necho '{{\"ok\": true}}'", log.display());
    executable(directory, "ssh", &body)
}

fn open_here(directory: &Path) -> std::io::Result<Vec<String>> {
    let program = executable(directory, "open-here", "exit 0")?;
    Ok(vec![program.display().to_string()])
}

fn edges(directory: &Path, seat_port: u16) -> std::io::Result<MacWakeEdges> {
    Ok(MacWakeEdges {
        avatar_host: "chise".to_owned(),
        room: Some("room-a".to_owned()),
        seat_port,
        ssh: fake_ssh(directory)?,
        launchctl: executable(directory, "launchctl", LAUNCHCTL)?,
        user_identifier: 501,
        open_command: open_here(directory)?,
        clock: SystemWaiting::new(),
    })
}

fn answer(listener: &TcpListener, body: &str) -> std::io::Result<()> {
    let (mut stream, _) = listener.accept()?;
    BufReader::new(&stream).read_line(&mut String::new())?;
    write!(stream, "HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n{body}")
}

fn health_server(body: &'static str) -> std::io::Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    thread::spawn(move || answer(&listener, body));
    Ok(port)
}

#[test]
fn commands_reach_the_avatar_host_over_batch_ssh_with_shell_quoting() -> TestResult {
    let directory = tempfile::tempdir()?;
    let words = ["avatar-listen", "read", "--since", "0 1"].map(str::to_owned);
    let output = edges(directory.path(), 1)?.at_the_avatar_host(&words, 15);
    let output = output.map_err(|failure| failure.message().to_owned())?;
    assert_eq!(output.trim(), "{\"ok\": true}");
    let log = logged(&directory.path().join("ssh.log"));
    assert_eq!(log.trim(), SSH_WORDS);
    Ok(())
}

#[test]
fn a_failing_ssh_is_an_unavailable_boundary() -> TestResult {
    let directory = tempfile::tempdir()?;
    let mut edges = edges(directory.path(), 1)?;
    edges.ssh = executable(directory.path(), "broken-ssh", "exit 255")?;
    let failure = edges.at_the_avatar_host(&["true".to_owned()], 5).err();
    assert!(failure.ok_or("expected a failure")?.is_unavailable());
    Ok(())
}

#[test]
fn the_conversation_opens_here_when_the_command_succeeds() -> TestResult {
    let directory = tempfile::tempdir()?;
    let mut edges = edges(directory.path(), 1)?;
    assert_eq!(edges.open_here(), Ok(true));
    edges.open_command = vec![
        executable(directory.path(), "no", "exit 1")?
            .display()
            .to_string(),
    ];
    assert_eq!(edges.open_here(), Ok(false));
    Ok(())
}

#[test]
fn listening_here_follows_the_launchd_agents() -> TestResult {
    let directory = tempfile::tempdir()?;
    assert!(edges(directory.path(), 1)?.listening_here());
    Ok(())
}

#[test]
fn the_seat_reports_whether_it_is_listening() -> TestResult {
    let directory = tempfile::tempdir()?;
    let port = health_server("{\"attached\": true, \"listening\": true}")?;
    assert!(edges(directory.path(), port)?.seat_listening());
    let silent = TcpListener::bind("127.0.0.1:0")?.local_addr()?.port();
    assert!(!edges(directory.path(), silent)?.seat_listening());
    Ok(())
}

#[test]
fn names_come_from_the_configuration() -> TestResult {
    let directory = tempfile::tempdir()?;
    let edges = edges(directory.path(), 1)?;
    assert_eq!(edges.avatar_host(), "chise");
    assert_eq!(edges.room_name().as_deref(), Some("room-a"));
    assert!(edges.clock() >= 0.0);
    Ok(())
}
