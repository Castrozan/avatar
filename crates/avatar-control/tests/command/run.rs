use std::time::Duration;

use avatar_control::command::{CommandEndpoints, run_command};
use axum::Router;
use axum::routing::get;
use serde_json::{Value, json};
use tokio::net::TcpListener;

use crate::server::{Server, acknowledging, only_identifying, serve, serve_on};
use crate::support::{TestResult, check};

fn words(arguments: &[&str]) -> Vec<String> {
    arguments.iter().map(|word| (*word).to_owned()).collect()
}

fn endpoints(server: &Server) -> CommandEndpoints {
    CommandEndpoints {
        control_url: server.url(),
        start_patience: Duration::from_millis(200),
        command_timeout: Duration::from_millis(300),
        ..base_endpoints()
    }
}

fn base_endpoints() -> CommandEndpoints {
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

struct Outcome {
    code: i32,
    stdout: String,
    stderr: String,
}

async fn run(arguments: &[&str], endpoints: &CommandEndpoints) -> Outcome {
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

async fn seat_server(body: &'static str) -> Result<String, Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let router = Router::new().route("/health", get(move || async move { body }));
    tokio::spawn(async move { drop(axum::serve(listener, router).await) });
    Ok(format!("http://{address}/health"))
}

async fn seat_url(
    body: Option<&'static str>,
    unreachable: String,
) -> Result<String, Box<dyn std::error::Error>> {
    match body {
        Some(text) => seat_server(text).await,
        None => Ok(unreachable),
    }
}

async fn spoken_output(
    seat_body: Option<&'static str>,
) -> Result<Value, Box<dyn std::error::Error>> {
    let server = serve(only_identifying).await?;
    let mut settings = endpoints(&server);
    settings.seat_health_url = seat_url(seat_body, settings.seat_health_url).await?;
    run(&["speak", "hello"], &settings).await;
    Ok(server.messages()[1].clone())
}

#[tokio::test]
async fn speech_goes_to_the_seat_when_a_mac_holds_it() -> TestResult {
    let attached = spoken_output(Some(r#"{"attached": true}"#)).await?;
    let detached = spoken_output(Some(r#"{"attached": false}"#)).await?;
    let unreachable = spoken_output(None).await?;
    let garbled = spoken_output(Some("not json")).await?;
    check(attached["output"] == "remote", &attached)?;
    let others = [detached, unreachable, garbled];
    check(
        others.iter().all(|command| command["output"] == "both"),
        json!(others),
    )
}

#[tokio::test]
async fn the_command_waits_for_a_server_that_is_still_starting() -> TestResult {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    drop(listener);
    let starter = tokio::spawn(start_late(address));
    let settings = late_endpoints(address);
    let outcome = run(&["status"], &settings).await;
    drop(starter.await?);
    let answered = outcome.code == 0 && outcome.stdout.contains("idle");
    check(answered, &outcome.stdout)
}

fn late_endpoints(address: std::net::SocketAddr) -> CommandEndpoints {
    CommandEndpoints {
        control_url: format!("ws://{address}"),
        ..base_endpoints()
    }
}

async fn start_late(address: std::net::SocketAddr) -> Result<Server, std::io::Error> {
    tokio::time::sleep(Duration::from_millis(300)).await;
    serve_on(TcpListener::bind(address).await?, acknowledging).await
}

#[tokio::test]
async fn a_refusing_server_is_given_up_on_after_the_patience() -> TestResult {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    drop(listener);
    let mut settings = late_endpoints(address);
    settings.start_patience = Duration::from_millis(100);
    let outcome = run(&["status"], &settings).await;
    check(
        outcome.code == 1 && !outcome.stderr.is_empty() && outcome.stdout.is_empty(),
        &outcome.stderr,
    )
}

#[tokio::test]
async fn a_server_that_hangs_up_without_answering_ends_quietly() -> TestResult {
    let server = serve(hanging_up).await?;
    let outcome = run(&["status"], &endpoints(&server)).await;
    check(
        outcome.code == 0 && outcome.stdout.is_empty() && outcome.stderr.is_empty(),
        &outcome.stderr,
    )
}

fn hanging_up(message: &Value) -> Vec<Value> {
    match message["type"].as_str() {
        Some("identify") => vec![json!({"type": "identifyAck"})],
        _ => vec![json!({"__close": true})],
    }
}
