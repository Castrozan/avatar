use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::time::Duration;

use avatar_control::control::MeterSettings;
use futures_util::StreamExt;
use serde_json::{Value, json};
use tokio::net::TcpStream;
use tokio::time::{sleep, timeout};
use tokio_tungstenite::tungstenite::Message;

use crate::rigs::start::{
    Failure, Socket, WAIT, check_flags, health, identify, identify_both, next_json, start,
};
use crate::support::{TestResult, check, fake_executable};

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
