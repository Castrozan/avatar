use std::fs;
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
use tokio::time::{sleep, timeout};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

use crate::support::{TestResult, check, fake_executable, http_request};

const WAIT: Duration = Duration::from_secs(10);

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;
type Failure = Box<dyn std::error::Error>;

fn ephemeral(name: &str) -> Option<String> {
    matches!(name, "AVATAR_WS_PORT" | "AVATAR_HTTP_PORT").then(|| "0".to_owned())
}

fn settings(home: &Path, meter: Option<MeterSettings>) -> std::io::Result<ControlServerSettings> {
    let environment = ControlEnvironment::from_lookup(ephemeral, home)?;
    let mut settings = ControlServerSettings::from_environment(environment);
    settings.meter = meter;
    Ok(settings)
}

async fn start(home: &Path, meter: Option<MeterSettings>) -> Result<ControlServer, Failure> {
    Ok(ControlServer::start(settings(home, meter)?, DiscardingSpeechPort).await?)
}

async fn identify(address: SocketAddr, role: &str) -> Result<Socket, Failure> {
    let (mut socket, _) = connect_async(format!("ws://{address}/")).await?;
    let request = json!({"type": "identify", "role": role}).to_string();
    socket.send(Message::Text(request.into())).await?;
    next_json(&mut socket).await?;
    Ok(socket)
}

async fn next_json(socket: &mut Socket) -> Result<Value, Failure> {
    match timeout(WAIT, socket.next()).await? {
        Some(Ok(Message::Text(text))) => Ok(serde_json::from_str(text.as_str())?),
        other => Err(format!("expected text, got {other:?}").into()),
    }
}

async fn health(address: SocketAddr) -> Result<Value, Failure> {
    let answer = http_request(address, "GET", "/health", None).await?;
    Ok(serde_json::from_str(&answer.body)?)
}

fn check_flags(before: &Value, after: &Value) -> TestResult {
    check(before["agentConnected"] == false, before)?;
    check(
        after["agentConnected"] == true && after["rendererConnected"] == true,
        after,
    )
}

async fn identify_both(address: SocketAddr) -> Result<(Socket, Socket), Failure> {
    Ok((
        identify(address, "agent").await?,
        identify(address, "renderer").await?,
    ))
}

#[tokio::test]
async fn health_follows_websocket_connections() -> TestResult {
    let home = tempfile::tempdir()?;
    let server = start(home.path(), None).await?;
    let before = health(server.http_address()).await?;
    let _clients = identify_both(server.websocket_address()).await?;
    let after = health(server.http_address()).await?;
    server.shutdown().await;
    check_flags(&before, &after)
}

fn playing_meter(home: &Path) -> Result<MeterSettings, Failure> {
    let samples: Vec<u8> = (0..480).flat_map(|_| 0.5_f32.to_le_bytes()).collect();
    fs::write(home.join("samples.bin"), samples)?;
    let script = format!("cat {}\nexec sleep 30", home.join("samples.bin").display());
    let program = fake_executable(home, "parec", &script)?;
    Ok(MeterSettings {
        program: program.to_string_lossy().into_owned(),
        source: "monitor".to_owned(),
        listen_again: Duration::from_millis(20),
        report_every: Duration::from_millis(10),
    })
}

#[tokio::test]
async fn microphone_levels_reach_the_renderer() -> TestResult {
    let home = tempfile::tempdir()?;
    let server = start(home.path(), Some(playing_meter(home.path())?)).await?;
    let mut renderer = identify(server.websocket_address(), "renderer").await?;
    let message = next_level_message(&mut renderer).await?;
    server.shutdown().await;
    check(message["type"] == "audioLevel", &message)
}

async fn next_level_message(renderer: &mut Socket) -> Result<Value, Failure> {
    let mut message = next_json(renderer).await?;
    while message["level"] != json!(0.5) {
        message = next_json(renderer).await?;
    }
    Ok(message)
}

async fn wait_until_refused(address: SocketAddr) {
    while TcpStream::connect(address).await.is_ok() {
        sleep(Duration::from_millis(25)).await;
    }
}

async fn refuses(address: SocketAddr) -> bool {
    timeout(WAIT, wait_until_refused(address)).await.is_ok()
}

async fn check_stopped(websocket: SocketAddr, http: SocketAddr, agent: &mut Socket) -> TestResult {
    let refused = refuses(websocket).await && refuses(http).await;
    let closed = timeout(WAIT, agent.next()).await?;
    let silent = !matches!(closed, Some(Ok(Message::Text(_))));
    check(refused && silent, format!("{refused} {closed:?}"))
}

#[tokio::test]
async fn shutdown_stops_both_listeners_and_closes_clients() -> TestResult {
    let home = tempfile::tempdir()?;
    let server = start(home.path(), None).await?;
    let (websocket, http) = (server.websocket_address(), server.http_address());
    let mut agent = identify(websocket, "agent").await?;
    server.shutdown().await;
    check_stopped(websocket, http, &mut agent).await
}
