use std::error::Error;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

use crate::TestResult;

pub(super) type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

const RECEIVE_TIMEOUT: Duration = Duration::from_secs(5);

pub(super) async fn send(socket: &mut Socket, message: &Value) -> TestResult {
    socket
        .send(Message::Text(message.to_string().into()))
        .await?;
    Ok(())
}

pub(super) async fn receive(socket: &mut Socket) -> Result<Value, Box<dyn Error>> {
    match timeout(RECEIVE_TIMEOUT, socket.next()).await? {
        Some(Ok(Message::Text(text))) => Ok(serde_json::from_str(text.as_str())?),
        other => Err(format!("expected a text frame, got {other:?}").into()),
    }
}

pub(super) async fn receive_kinds(
    socket: &mut Socket,
    count: usize,
) -> Result<Vec<Value>, Box<dyn Error>> {
    let mut messages = Vec::new();
    for _ in 0..count {
        messages.push(receive(socket).await?);
    }
    Ok(messages)
}

pub(super) async fn identified(url: &str, role: &str) -> Result<Socket, Box<dyn Error>> {
    let (mut socket, _) = connect_async(url).await?;
    send(&mut socket, &json!({"type": "identify", "role": role})).await?;
    let acknowledgement = receive(&mut socket).await?;
    assert_eq!(acknowledgement["type"], "identifyAck");
    Ok(socket)
}

pub(super) fn kinds(messages: &[Value]) -> Vec<&str> {
    messages
        .iter()
        .filter_map(|message| message["type"].as_str())
        .collect()
}
