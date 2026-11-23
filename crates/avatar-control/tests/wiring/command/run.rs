use std::time::Duration;

use avatar_control::command::{CommandEndpoints, run_command};
use serde_json::{Value, json};

use crate::script::{acknowledging, only_identifying};
use crate::server::{Server, serve};
use crate::support::{TestResult, check};

pub(crate) fn words(arguments: &[&str]) -> Vec<String> {
    arguments.iter().map(|word| (*word).to_owned()).collect()
}

pub(crate) fn endpoints(server: &Server) -> CommandEndpoints {
    CommandEndpoints {
        control_url: server.url(),
        start_patience: Duration::from_millis(200),
        command_timeout: Duration::from_millis(300),
        ..base_endpoints()
    }
}

pub(crate) fn base_endpoints() -> CommandEndpoints {
    CommandEndpoints {
        control_url: String::new(),
        seat_health_url: "http://127.0.0.1:1/health".to_owned(),
        seat_patience: Duration::from_millis(300),
        start_patience: Duration::from_secs(10),
        start_retry: Duration::from_millis(20),
        speech_timeout: Duration::from_millis(200),
        command_timeout: Duration::from_secs(5),
    }
}

pub(crate) struct Outcome {
    pub(crate) code: i32,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

pub(crate) async fn run(arguments: &[&str], endpoints: &CommandEndpoints) -> Outcome {
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    let words = words(arguments);
    let code = run_command(&words, endpoints, "id-1", &mut stdout, &mut stderr).await;
    Outcome {
        code,
        stdout: String::from_utf8_lossy(&stdout).into_owned(),
        stderr: String::from_utf8_lossy(&stderr).into_owned(),
    }
}

#[tokio::test]
async fn a_status_command_prints_the_answer_and_sends_the_identified_command() -> TestResult {
    let server = serve(acknowledging).await?;
    let outcome = run(&["status"], &endpoints(&server)).await;
    let printed: Value = serde_json::from_str(outcome.stdout.trim())?;
    check(
        outcome.code == 0 && printed["state"] == "idle",
        &outcome.stdout,
    )?;
    let identify = json!({"type": "identify", "role": "agent"});
    let status = json!({"type": "getStatus", "id": "id-1"});
    check(server.messages() == [identify, status], "sent messages")
}

#[tokio::test]
async fn answers_for_other_requests_are_ignored_until_the_matching_one() -> TestResult {
    let server = serve(acknowledging).await?;
    let outcome = run(&["character", "Aiko"], &endpoints(&server)).await;
    let printed: Value = serde_json::from_str(outcome.stdout.trim())?;
    check(
        outcome.code == 0 && printed["id"] == "id-1",
        &outcome.stdout,
    )?;
    check(outcome.stdout.lines().count() == 1, &outcome.stdout)
}

#[tokio::test]
async fn error_answers_set_a_failing_exit_code() -> TestResult {
    let server = serve(acknowledging).await?;
    let mode = run(&["mode", "camera"], &endpoints(&server)).await;
    let stop = run(&["stop-speaking"], &endpoints(&server)).await;
    let mode_failed = mode.code == 1 && mode.stdout.contains("videoModeError");
    let stop_failed = stop.code == 1 && stop.stdout.contains("\"error\"");
    check(
        mode_failed && stop_failed,
        format!("{} {}", mode.stdout, stop.stdout),
    )
}

#[tokio::test]
async fn usage_errors_never_connect() -> TestResult {
    let server = serve(acknowledging).await?;
    let outcome = run(&["bogus"], &endpoints(&server)).await;
    check(
        outcome.code == 1 && outcome.stderr.starts_with("Usage: avatar-command"),
        &outcome.stderr,
    )?;
    check(server.messages().is_empty(), "connected anyway")
}

#[tokio::test]
async fn an_unanswered_command_times_out() -> TestResult {
    let server = serve(only_identifying).await?;
    let outcome = run(&["status"], &endpoints(&server)).await;
    check(
        outcome.code == 1 && outcome.stderr == "Avatar command timed out\n",
        &outcome.stderr,
    )
}
