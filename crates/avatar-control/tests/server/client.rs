use std::error::Error;
use std::net::SocketAddr;
use std::time::Duration;

use avatar_control::server::{DiscardingSpeechPort, ShutdownHandle, serve};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

use crate::TestResult;

const RECEIVE_TIMEOUT: Duration = Duration::from_secs(5);

pub(crate) type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub(crate) async fn start() -> Result<(String, ShutdownHandle), Box<dyn Error>> {
    let loopback = SocketAddr::from(([127, 0, 0, 1], 0));
    let (address, shutdown) = serve(loopback, DiscardingSpeechPort).await?;
    Ok((format!("ws://{address}/"), shutdown))
}

pub(crate) async fn connect(url: &str) -> Result<Socket, Box<dyn Error>> {
    let (socket, _) = connect_async(url).await?;
    Ok(socket)
}

pub(crate) async fn send_text(socket: &mut Socket, text: &str) -> TestResult {
    socket.send(Message::Text(text.into())).await?;
    Ok(())
}

pub(crate) async fn next_json(socket: &mut Socket) -> Result<Value, Box<dyn Error>> {
    match timeout(RECEIVE_TIMEOUT, socket.next()).await? {
        Some(Ok(Message::Text(text))) => Ok(serde_json::from_str(text.as_str())?),
        other => Err(format!("expected a text frame, got {other:?}").into()),
    }
}

pub(crate) async fn close_and_wait(socket: &mut Socket) -> TestResult {
    let _ = socket.close(None).await;
    while let Some(Ok(_)) = timeout(RECEIVE_TIMEOUT, socket.next()).await? {}
    Ok(())
}
