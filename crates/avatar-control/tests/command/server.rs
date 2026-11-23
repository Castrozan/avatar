use std::io;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, PoisonError};

use axum::Router;
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use serde_json::{Value, json};
use tokio::net::TcpListener;

pub(crate) type Script = fn(&Value) -> Vec<Value>;
pub(crate) type Received = Arc<Mutex<Vec<Value>>>;

#[derive(Clone)]
struct ServerState {
    script: Script,
    received: Received,
}

impl ServerState {
    fn new(script: Script, received: Received) -> Self {
        Self { script, received }
    }
}

pub(crate) struct Server {
    pub(crate) address: SocketAddr,
    pub(crate) received: Received,
}

impl Server {
    pub(crate) fn url(&self) -> String {
        format!("ws://{}", self.address)
    }

    pub(crate) fn messages(&self) -> Vec<Value> {
        self.received
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

pub(crate) async fn serve_on(listener: TcpListener, script: Script) -> io::Result<Server> {
    let address = listener.local_addr()?;
    let received = Received::default();
    let state = ServerState::new(script, Arc::clone(&received));
    let router = Router::new().route("/", get(upgrade)).with_state(state);
    tokio::spawn(run_router(listener, router));
    Ok(Server { address, received })
}

async fn run_router(listener: TcpListener, router: Router) {
    drop(axum::serve(listener, router).await);
}

pub(crate) async fn serve(script: Script) -> io::Result<Server> {
    serve_on(TcpListener::bind("127.0.0.1:0").await?, script).await
}

async fn upgrade(State(state): State<ServerState>, upgrade: WebSocketUpgrade) -> impl IntoResponse {
    upgrade.on_upgrade(move |socket| converse(socket, state))
}

async fn converse(mut socket: WebSocket, state: ServerState) {
    while let Some(Ok(Message::Text(text))) = socket.recv().await
        && answer(&mut socket, &state, text.as_str()).await
    {}
}

fn record(received: &Received, message: &Value) {
    received
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .push(message.clone());
}

async fn answer(socket: &mut WebSocket, state: &ServerState, text: &str) -> bool {
    let Ok(message) = serde_json::from_str::<Value>(text) else {
        return true;
    };
    record(&state.received, &message);
    let replies = (state.script)(&message);
    deliver(socket, &replies).await
}

async fn deliver(socket: &mut WebSocket, replies: &[Value]) -> bool {
    let mut open = true;
    for reply in replies {
        open = open && deliver_one(socket, reply).await;
    }
    open
}

async fn deliver_one(socket: &mut WebSocket, reply: &Value) -> bool {
    if reply.get("__close").is_some() {
        return false;
    }
    drop(socket.send(Message::Text(reply.to_string().into())).await);
    true
}

fn character_changes(message: &Value) -> Vec<Value> {
    vec![
        json!({"type": "characterChanged", "id": "stale"}),
        json!({"type": "characterChanged", "id": message["id"], "name": message["name"]}),
    ]
}

pub(crate) fn acknowledging(message: &Value) -> Vec<Value> {
    match message["type"].as_str() {
        Some("identify") => vec![json!({"type": "identifyAck", "role": "agent"})],
        Some("getStatus") => vec![json!({"type": "status", "state": "idle"})],
        Some("setCharacter") => character_changes(message),
        Some("setVideoMode") => vec![json!({"type": "videoModeError", "id": message["id"]})],
        Some("stopSpeaking") => vec![json!({"type": "error", "error": "bad"})],
        _ => Vec::new(),
    }
}

pub(crate) fn only_identifying(message: &Value) -> Vec<Value> {
    match message["type"].as_str() {
        Some("identify") => vec![json!({"type": "identifyAck"})],
        _ => Vec::new(),
    }
}
