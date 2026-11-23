use serde_json::{Value, json};

use crate::TestResult;
use crate::client::{Socket, close_and_wait, connect, next_json, send_text, start};
use crate::ensure;

#[tokio::test]
async fn identify_agent_gets_an_ack_over_the_socket() -> TestResult {
    let (url, shutdown) = start().await?;
    let ack = identify(&url, "agent").await;
    shutdown.shutdown();
    let expected = json!({"type": "identifyAck", "role": "agent", "status": "connected", "serverVersion": "1.0.0"});
    ensure(ack? == expected, || "agent ack differs".to_owned())
}

#[tokio::test]
async fn server_survives_a_renderer_disconnect() -> TestResult {
    let (url, shutdown) = start().await?;
    let outcome = renderer_leaves_then_agent_identifies(&url).await;
    shutdown.shutdown();
    outcome
}

async fn renderer_leaves_then_agent_identifies(url: &str) -> TestResult {
    renderer_joins_and_leaves(url).await?;
    let ack = identify(url, "agent").await?;
    ensure(ack["type"] == "identifyAck", || {
        format!("after disconnect got {ack}")
    })
}

async fn renderer_joins_and_leaves(url: &str) -> TestResult {
    let mut renderer = connect(url).await?;
    send_text(&mut renderer, r#"{"type":"identify","role":"renderer"}"#).await?;
    for _ in 0..2 {
        next_json(&mut renderer).await?;
    }
    close_and_wait(&mut renderer).await
}

async fn identify(url: &str, role: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let mut socket: Socket = connect(url).await?;
    send_text(
        &mut socket,
        &format!(r#"{{"type":"identify","role":"{role}"}}"#),
    )
    .await?;
    next_json(&mut socket).await
}
