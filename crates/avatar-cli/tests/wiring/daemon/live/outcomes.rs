use std::fs;
use std::io::Read;
use std::process::Child;
use std::time::Duration;

use nix::sys::signal::{Signal, kill};
use nix::unistd::Pid;

use super::server::MockLive;
use super::{A2A, live_machine_with, run_session, spawn_live};
use crate::daemon::waiting::{eventually, exit_code_within};
use crate::{Machine, TestResult};

const A2A_LOG: &str = "echo \"$@\" >> \"${0%/*}/../a2a.log\"";
const REFUSING: &str =
    "case \"$1\" in send) echo 'a2a: peer refused the task'; exit 1;; notify) echo '{}';; esac";
const NEVER_ANSWERING: &str = "case \"$1\" in send) echo task-1;; status) echo '{\"state\":\"working\",\"output\":\"\"}';; notify) echo '{}';; esac";
const REFUSED_NOTICE: &str = "notify --sender avatar live voice pane-1 Your answer to LIVE-ANSWER-1 never reached Lucas (a2a: peer refused the task); answer it again now with a closing line LIVE-ANSWER-1: <one sentence>.";

fn has_polled_the_task(machine: &Machine) -> bool {
    let log = fs::read_to_string(machine.path("a2a.log")).unwrap_or_default();
    log.contains("status pane-1 task-1")
}

fn first_poll_seen(machine: &Machine) -> bool {
    eventually(|| has_polled_the_task(machine), Duration::from_secs(30))
}

fn terminate(child: Child) -> TestResult {
    kill(Pid::from_raw(i32::try_from(child.id())?), Signal::SIGTERM)?;
    assert!(exit_code_within(child, Duration::from_mins(1))?.is_some());
    Ok(())
}

fn stdout_after_sigterm(machine: &Machine) -> Result<String, Box<dyn std::error::Error>> {
    let server = MockLive::start()?;
    let mut child = spawn_live(machine, &format!("ws://{}", server.address))?;
    let mut screen = child.stdout.take().ok_or("the daemon has no stdout")?;
    assert!(first_poll_seen(machine));
    terminate(child)?;
    let mut logged = String::new();
    screen.read_to_string(&mut logged)?;
    Ok(logged)
}

#[test]
fn an_answer_is_logged_with_the_task_number_it_answers() -> TestResult {
    let session = run_session(A2A)?;
    assert!(session.screen().contains("task 1 answered by servant in "));
    Ok(())
}

#[test]
fn a_refused_task_is_logged_as_failed_and_the_agent_is_told() -> TestResult {
    let session = run_session(&format!("{A2A_LOG}; {REFUSING}"))?;
    let screen = session.screen();
    assert!(screen.contains("task 1 failed for servant after "));
    assert!(screen.contains(" s: a2a: peer refused the task"));
    assert!(screen.contains("task 1 notice sent to servant: a2a: peer refused the task"));
    assert!(session.file("a2a.log").contains(REFUSED_NOTICE));
    Ok(())
}

#[test]
fn a_task_pending_at_sigterm_is_logged_as_unanswered_and_the_agent_is_told() -> TestResult {
    let machine = live_machine_with(&format!("{A2A_LOG}; {NEVER_ANSWERING}"))?;
    let logged = stdout_after_sigterm(&machine)?;
    assert!(logged.contains("task 1 unanswered for servant after "));
    assert!(logged.contains(" s: the live voice stopped"));
    assert!(logged.contains("task 1 notice sent to servant: the live voice stopped"));
    Ok(())
}
