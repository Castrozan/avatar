use std::error::Error;
use std::fs;

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;

use super::seat_world::{LOG, Seat, finished_within_patience, wait_for};

type TestResult = Result<(), Box<dyn Error>>;

fn run_to_the_end(seat: &Seat, query: &[&str]) -> Result<bool, Box<dyn Error>> {
    Ok(finished_within_patience(&mut seat.holder(query).spawn()?))
}

fn write_log(seat: &Seat, text: &str) -> TestResult {
    fs::create_dir_all(seat.path(LOG).parent().ok_or("no parent")?)?;
    Ok(fs::write(seat.path(LOG), text)?)
}

#[test]
fn a_tunnel_that_ends_ends_the_seat_and_logs_its_exit_status() -> TestResult {
    let seat = Seat::with_ssh("echo \"$@\" > \"$HOME/ssh-arguments\"\nexit 3")?;
    assert!(run_to_the_end(&seat, &[""])?);
    let arguments = fs::read_to_string(seat.path("ssh-arguments"))?;
    assert!(arguments.contains("-L 127.0.0.1:8769:127.0.0.1:8769"));
    assert!(arguments.contains("avatar-remote-seat.service"));
    assert!(seat.log().contains("its tunnel ended with 3"));
    Ok(())
}

#[test]
fn a_termination_request_ends_the_seat_and_its_tunnel() -> TestResult {
    let seat = Seat::with_ssh("touch \"$HOME/tunnel-up\"\nexec sleep 60")?;
    let mut child = seat.holder(&["?meeting"]).spawn()?;
    wait_for(&seat.path("tunnel-up"));
    kill(Pid::from_raw(i32::try_from(child.id())?), Signal::SIGTERM)?;
    assert!(finished_within_patience(&mut child));
    assert!(seat.log().contains("its tunnel ended with None"));
    Ok(())
}

#[test]
fn a_short_log_is_appended_to() -> TestResult {
    let seat = Seat::with_ssh("exit 0")?;
    write_log(&seat, "earlier\n")?;
    assert!(run_to_the_end(&seat, &[])?);
    assert!(seat.log().starts_with("earlier\n"));
    Ok(())
}

#[test]
fn a_log_past_a_million_bytes_starts_over() -> TestResult {
    let seat = Seat::with_ssh("exit 0")?;
    write_log(&seat, &"x".repeat(1_000_001))?;
    assert!(run_to_the_end(&seat, &[])?);
    let restarted = seat.log();
    assert!(restarted.len() < 1_000_000 && !restarted.contains("xxxx"));
    assert!(restarted.contains("its tunnel ended with 0"));
    Ok(())
}
