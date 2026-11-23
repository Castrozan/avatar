use std::time::Duration;

use avatar_control::command::CommandEndpoints;
use serde_json::{Value, json};
use tokio::net::TcpListener;

use crate::run::{base_endpoints, endpoints, run};
use crate::script::acknowledging;
use crate::server::{Server, serve, serve_on};
use crate::support::{TestResult, check};

#[tokio::test]
pub(crate) async fn the_command_waits_for_a_server_that_is_still_starting() -> TestResult {
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

pub(crate) fn late_endpoints(address: std::net::SocketAddr) -> CommandEndpoints {
    CommandEndpoints {
        control_url: format!("ws://{address}"),
        ..base_endpoints()
    }
}

pub(crate) async fn start_late(address: std::net::SocketAddr) -> Result<Server, std::io::Error> {
    tokio::time::sleep(Duration::from_millis(300)).await;
    serve_on(TcpListener::bind(address).await?, acknowledging).await
}

#[tokio::test]
pub(crate) async fn a_refusing_server_is_given_up_on_after_the_patience() -> TestResult {
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
pub(crate) async fn a_server_that_hangs_up_without_answering_ends_quietly() -> TestResult {
    let server = serve(hanging_up).await?;
    let outcome = run(&["status"], &endpoints(&server)).await;
    check(
        outcome.code == 0 && outcome.stdout.is_empty() && outcome.stderr.is_empty(),
        &outcome.stderr,
    )
}

pub(crate) fn hanging_up(message: &Value) -> Vec<Value> {
    match message["type"].as_str() {
        Some("identify") => vec![json!({"type": "identifyAck"})],
        _ => vec![json!({"__close": true})],
    }
}
