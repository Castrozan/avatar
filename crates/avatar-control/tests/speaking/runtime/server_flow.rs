use std::error::Error;
use std::net::SocketAddr;
use std::time::Duration;

use avatar_control::server::{ShutdownHandle, serve};
use avatar_control::session::control_origins;
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

use super::harness::launch;
use crate::TestResult;

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

const RECEIVE_TIMEOUT: Duration = Duration::from_secs(5);

async fn send(socket: &mut Socket, message: &Value) -> TestResult {
    socket
        .send(Message::Text(message.to_string().into()))
        .await?;
    Ok(())
}

async fn receive(socket: &mut Socket) -> Result<Value, Box<dyn Error>> {
    match timeout(RECEIVE_TIMEOUT, socket.next()).await? {
        Some(Ok(Message::Text(text))) => Ok(serde_json::from_str(text.as_str())?),
        other => Err(format!("expected a text frame, got {other:?}").into()),
    }
}

async fn receive_kinds(socket: &mut Socket, count: usize) -> Result<Vec<Value>, Box<dyn Error>> {
    let mut messages = Vec::new();
    for _ in 0..count {
        messages.push(receive(socket).await?);
    }
    Ok(messages)
}

async fn identified(url: &str, role: &str) -> Result<Socket, Box<dyn Error>> {
    let (mut socket, _) = connect_async(url).await?;
    send(&mut socket, &json!({"type": "identify", "role": role})).await?;
    let acknowledgement = receive(&mut socket).await?;
    assert_eq!(acknowledgement["type"], "identifyAck");
    Ok(socket)
}

fn kinds(messages: &[Value]) -> Vec<&str> {
    messages
        .iter()
        .filter_map(|message| message["type"].as_str())
        .collect()
}

async fn start() -> Result<(String, ShutdownHandle, tempfile::TempDir), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let (port, _world) = launch(&json!({"steps": []}), directory.path())?;
    let loopback = SocketAddr::from(([127, 0, 0, 1], 0));
    let (address, shutdown) = serve(loopback, port, control_origins("3000")).await?;
    Ok((format!("ws://{address}/"), shutdown, directory))
}

async fn follow(agent: &mut Socket) -> TestResult {
    send(agent, &json!({"type": "followSpeech"})).await?;
    let initial = receive(agent).await?;
    assert_eq!(
        initial,
        json!({"type": "speechActivity", "speaking": false})
    );
    Ok(())
}

async fn exchange(
    agent: &mut Socket,
    renderer: &mut Socket,
) -> Result<(Vec<Value>, Vec<Value>), Box<dyn Error>> {
    let command = json!({"type": "speak", "text": "Hello", "id": "flow"});
    send(agent, &command).await?;
    Ok((
        receive_kinds(agent, 3).await?,
        receive_kinds(renderer, 3).await?,
    ))
}

async fn speak_as_follower(url: &str) -> Result<(Vec<Value>, Vec<Value>), Box<dyn Error>> {
    let mut renderer = identified(url, "renderer").await?;
    let mut agent = identified(url, "agent").await?;
    follow(&mut agent).await?;
    exchange(&mut agent, &mut renderer).await
}

fn expected_exchange() -> Value {
    json!({
        "heard": ["speechActivity", "speechActivity", "speakAck"],
        "status": "finished",
        "shown": ["initialState", "startSpeaking", "speechEnded"],
    })
}

fn observed_exchange(heard: &[Value], shown: &[Value]) -> Value {
    json!({"heard": kinds(heard), "status": heard[2]["status"], "shown": kinds(shown)})
}

#[tokio::test]
async fn speech_reaches_the_requester_the_renderer_and_followers_through_the_server() -> TestResult
{
    let (url, shutdown, _directory) = start().await?;
    let (heard, shown) = speak_as_follower(&url).await?;
    shutdown.shutdown();
    assert_eq!(observed_exchange(&heard, &shown), expected_exchange());
    Ok(())
}
