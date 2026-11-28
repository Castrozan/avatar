use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::Response;
use axum::routing::get;
use serde_json::Value;
use tokio::net::TcpListener;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

pub(super) const CLOSE: &str = "\"close\"";

struct Script {
    replies: Vec<String>,
    received: UnboundedSender<Value>,
}

pub(super) struct MockServer {
    pub(super) address: SocketAddr,
    pub(super) received: UnboundedReceiver<Value>,
}

async fn upgrade(State(script): State<Arc<Script>>, socket: WebSocketUpgrade) -> Response {
    socket.on_upgrade(move |connection| serve(connection, script))
}

fn frame_of(message: &str) -> Message {
    if message == CLOSE {
        return Message::Close(None);
    }
    Message::Text(message.into())
}

async fn reply<'a>(socket: &mut WebSocket, replies: impl Iterator<Item = &'a String>) {
    for message in replies {
        socket.send(frame_of(message)).await.ok();
    }
}

fn record(script: &Script, message: &str) {
    script
        .received
        .send(serde_json::from_str(message).unwrap_or_default())
        .ok();
}

async fn serve(mut socket: WebSocket, script: Arc<Script>) {
    let mut answered = false;
    while let Some(Ok(Message::Text(message))) = socket.recv().await {
        record(&script, message.as_str());
        reply(&mut socket, script.replies.iter().filter(|_| !answered)).await;
        answered = true;
    }
}

async fn listen(script: Arc<Script>) -> Result<SocketAddr, std::io::Error> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let application = Router::new().route("/", get(upgrade)).with_state(script);
    tokio::spawn(async move { axum::serve(listener, application).await });
    Ok(address)
}

fn script_of(replies: &[Value], received: UnboundedSender<Value>) -> Script {
    Script {
        replies: replies.iter().map(Value::to_string).collect(),
        received,
    }
}

pub(super) async fn start(replies: &[Value]) -> Result<MockServer, std::io::Error> {
    let (received, inbox) = unbounded_channel();
    let address = listen(Arc::new(script_of(replies, received))).await?;
    Ok(MockServer {
        address,
        received: inbox,
    })
}
