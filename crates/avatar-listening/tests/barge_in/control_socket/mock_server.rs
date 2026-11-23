use std::error::Error;
use std::future::IntoFuture;
use std::time::Duration;

use axum::Router;
use axum::extract::State;
use axum::extract::ws::WebSocketUpgrade;
use axum::response::Response;
use axum::routing::get;
use serde_json::Value;
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc};
use tokio::time::timeout;

mod relay;

pub(super) const WAIT: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub(super) enum MockCommand {
    Notice(String),
    Close,
}

#[derive(Clone)]
struct MockState {
    received: mpsc::UnboundedSender<String>,
    commands: broadcast::Sender<MockCommand>,
}

pub(super) struct MockControlServer {
    pub(super) url: String,
    received: mpsc::UnboundedReceiver<String>,
    commands: broadcast::Sender<MockCommand>,
}

impl MockControlServer {
    pub(super) async fn start() -> Result<Self, Box<dyn Error>> {
        let (listener, url) = bind_loopback().await?;
        let (received_sender, received) = mpsc::unbounded_channel();
        let commands = serve_in_background(listener, received_sender);
        Ok(Self {
            url,
            received,
            commands,
        })
    }

    pub(super) fn send(&self, command: MockCommand) {
        self.commands.send(command).ok();
    }

    pub(super) async fn next_message(&mut self) -> Result<Value, Box<dyn Error>> {
        let text = timeout(WAIT, self.received.recv())
            .await?
            .ok_or("mock server closed")?;
        Ok(serde_json::from_str(&text)?)
    }
}

fn serve_in_background(
    listener: TcpListener,
    received_sender: mpsc::UnboundedSender<String>,
) -> broadcast::Sender<MockCommand> {
    let (commands, _) = broadcast::channel(16);
    let router = router(received_sender, commands.clone());
    tokio::spawn(axum::serve(listener, router).into_future());
    commands
}

async fn bind_loopback() -> Result<(TcpListener, String), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let url = format!("ws://{}", listener.local_addr()?);
    Ok((listener, url))
}

fn router(
    received: mpsc::UnboundedSender<String>,
    commands: broadcast::Sender<MockCommand>,
) -> Router {
    Router::new()
        .route("/", get(upgrade))
        .with_state(MockState { received, commands })
}

async fn upgrade(State(state): State<MockState>, websocket: WebSocketUpgrade) -> Response {
    let received = state.received.clone();
    let commands = state.commands.subscribe();
    websocket.on_upgrade(move |socket| relay::serve(socket, received, commands))
}
