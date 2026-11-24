use std::error::Error;
use std::io::{BufRead, BufReader};
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

use super::{Machine, STATUS, TestResult, configured, mac_talk, ssh_log, ssh_machine, text};

const HEARD: &str = "[voice] hello";
const UNTIL_INPUT_CLOSES: &str = "while read -r line; do :; done";
const CLOSED: &str = "{\"open\": false, \"live\": false, \"seat\": null, \"last_heard\": 0}";
const LOOKED: &str = "-o BatchMode=yes -o ConnectTimeout=10 chise avatar-talk status";
const PATIENCE: Duration = Duration::from_secs(20);

fn within_patience<Found>(mut look: impl FnMut() -> Option<Found>) -> Option<Found> {
    let deadline = Instant::now() + PATIENCE;
    std::iter::from_fn(|| (Instant::now() < deadline).then(|| look().or_else(pause)))
        .flatten()
        .next()
}

fn pause<Found>() -> Option<Found> {
    sleep(Duration::from_millis(10));
    None
}

fn ended(talk: &mut Child) -> Option<ExitStatus> {
    let status = within_patience(|| talk.try_wait().ok().flatten());
    let _ = talk.kill();
    status
}

fn appeared(path: &Path) -> bool {
    within_patience(|| path.exists().then_some(())).is_some()
}

fn room_that_speaks_then_waits(machine: &Machine) -> Result<PathBuf, Box<dyn Error>> {
    let released = machine.path("released");
    let let_go = format!("echo gone > {}", released.display());
    let room = format!("echo '{HEARD}'\n{UNTIL_INPUT_CLOSES}\n{let_go}");
    machine.install("ssh", &room)?;
    Ok(released)
}

fn following(machine: &Machine) -> Command {
    let mut command = configured(machine, &["follow"]);
    command.env("AVATAR_ROOM", "elsewhere");
    command
}

fn first_line(talk: &mut Child) -> Result<String, Box<dyn Error>> {
    let mut line = String::new();
    BufReader::new(talk.stdout.take().ok_or("no stdout")?).read_line(&mut line)?;
    Ok(line.trim().to_owned())
}

fn status_through(machine: &Machine, room: &str) -> Result<Output, Box<dyn Error>> {
    machine.install("ssh", room)?;
    mac_talk(machine, &["status"])
}

#[test]
fn a_forwarded_follow_prints_each_line_as_it_arrives_and_lets_go_when_it_is_stopped() -> TestResult
{
    let machine = Machine::with_tools(&[])?;
    let released = room_that_speaks_then_waits(&machine)?;
    let mut talk = following(&machine).stdout(Stdio::piped()).spawn()?;
    let heard = first_line(&mut talk)?;
    assert_eq!((heard.as_str(), talk.try_wait()?), (HEARD, None));
    talk.kill()?;
    assert!(appeared(&released));
    Ok(())
}

#[test]
fn a_listening_command_ends_when_the_machine_that_sent_it_lets_go_of_its_input() -> TestResult {
    let machine = Machine::with_tools(&[("ssh", UNTIL_INPUT_CLOSES)])?;
    let mut command = following(&machine);
    command.env("AVATAR_CALLER_HOLDS_INPUT", "1");
    let mut talk = command.stdin(Stdio::piped()).spawn()?;
    drop(talk.stdin.take());
    let signal = ended(&mut talk).and_then(|status| status.signal());
    assert_eq!(signal, Some(15));
    Ok(())
}

#[test]
fn after_the_host_opens_a_seat_elsewhere_the_plain_commands_reach_it_until_it_closes() -> TestResult
{
    let machine = ssh_machine("exit 0")?;
    mac_talk(&machine, &["open", "--seat", "discord"])?;
    let reached = mac_talk(&machine, &["status"])?;
    assert_eq!(text(&reached.stdout).trim(), STATUS);
    assert_eq!(ssh_log(&machine)?, LOOKED);
    let closed = status_through(&machine, &format!("echo '{CLOSED}'"))?;
    assert!(text(&closed.stderr).contains("opened on chise is closed"));
    status_through(&machine, "echo reached > $HOME/reached-again")?;
    assert!(!machine.path("reached-again").exists());
    Ok(())
}
