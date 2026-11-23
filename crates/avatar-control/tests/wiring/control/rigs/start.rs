use std::net::SocketAddr;
use std::path::Path;
use std::time::Duration;

use avatar_control::control::{
    ControlEnvironment, ControlServer, ControlServerSettings, MeterSettings,
};
use avatar_control::server::DiscardingSpeechPort;
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

use crate::support::{TestResult, check, http_request};

pub(crate) const WAIT: Duration = Duration::from_secs(10);

pub(crate) type Failure = Box<dyn std::error::Error>;
pub(crate) type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub(crate) fn ephemeral(name: &str) -> Option<String> {
    matches!(name, "AVATAR_WS_PORT" | "AVATAR_HTTP_PORT").then(|| "0".to_owned())
}

pub(crate) fn settings(
    home: &Path,
    meter: Option<MeterSettings>,
) -> std::io::Result<ControlServerSettings> {
    let environment = ControlEnvironment::from_lookup(ephemeral, home)?;
    let mut settings = ControlServerSettings::from_environment(environment);
    settings.meter = meter;
    Ok(settings)
}

pub(crate) async fn start(
    home: &Path,
    meter: Option<MeterSettings>,
) -> Result<ControlServer, Failure> {
    Ok(ControlServer::start(settings(home, meter)?, DiscardingSpeechPort).await?)
}

pub(crate) async fn identify(address: SocketAddr, role: &str) -> Result<Socket, Failure> {
    let (mut socket, _) = connect_async(format!("ws://{address}/")).await?;
    let request = json!({"type": "identify", "role": role}).to_string();
    socket.send(Message::Text(request.into())).await?;
    next_json(&mut socket).await?;
    Ok(socket)
}

pub(crate) async fn next_json(socket: &mut Socket) -> Result<Value, Failure> {
    match timeout(WAIT, socket.next()).await? {
        Some(Ok(Message::Text(text))) => Ok(serde_json::from_str(text.as_str())?),
        other => Err(format!("expected text, got {other:?}").into()),
    }
}

pub(crate) async fn health(address: SocketAddr) -> Result<Value, Failure> {
    let answer = http_request(address, "GET", "/health", None).await?;
    Ok(serde_json::from_str(&answer.body)?)
}

pub(crate) fn check_flags(before: &Value, after: &Value) -> TestResult {
    check(before["agentConnected"] == false, before)?;
    check(
        after["agentConnected"] == true && after["rendererConnected"] == true,
        after,
    )
}

pub(crate) async fn identify_both(address: SocketAddr) -> Result<(Socket, Socket), Failure> {
    Ok((
        identify(address, "agent").await?,
        identify(address, "renderer").await?,
    ))
}
