use std::io;
use std::net::SocketAddr;
use std::sync::mpsc::{Receiver, Sender, channel};

use std::sync::{Arc, Mutex, PoisonError};
use tokio::net::TcpListener;

use axum::Router;
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::{HeaderMap, Method, Uri};
use axum::response::Response;
use axum::routing::get;
use serde_json::Value;

use super::heard::Heard;
use super::script::{frame_of, replies_to, rest_reply};
use super::server_thread::spawned;

pub(super) struct Channels {
    pub(super) keys: Sender<String>,
    pub(super) messages: Sender<Value>,
}

struct Recorder {
    address: SocketAddr,
    channels: Mutex<Channels>,
}

type Shared = Arc<Recorder>;

fn channels_of(recorder: &Recorder) -> std::sync::MutexGuard<'_, Channels> {
    recorder
        .channels
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

async fn rest(State(recorder): State<Shared>, request: (Method, Uri, HeaderMap)) -> String {
    let (method, uri, headers) = request;
    let key = headers
        .get("xi-api-key")
        .and_then(|value| value.to_str().ok());
    let _ = channels_of(&recorder)
        .keys
        .send(key.unwrap_or_default().to_owned());
    rest_reply(&method, uri.path(), recorder.address).to_string()
}

async fn send_all(socket: &mut WebSocket, replies: Vec<Value>) {
    for reply in replies {
        let _ = socket.send(frame_of(&reply)).await;
    }
}

async fn converse(mut socket: WebSocket, recorder: Shared) {
    while let Some(Ok(Message::Text(text))) = socket.recv().await {
        let message: Value = serde_json::from_str(text.as_str()).unwrap_or_default();
        let replies = replies_to(&message);
        let _ = channels_of(&recorder).messages.send(message);
        send_all(&mut socket, replies).await;
    }
}

async fn upgrade(State(recorder): State<Shared>, socket: WebSocketUpgrade) -> Response {
    socket.on_upgrade(move |connection| converse(connection, recorder))
}

pub(super) async fn serve(listener: TcpListener, address: SocketAddr, channels: Channels) {
    let channels = Mutex::new(channels);
    let recorder = Arc::new(Recorder { address, channels });
    let routes = Router::new().route("/socket", get(upgrade)).fallback(rest);
    let _ = axum::serve(listener, routes.with_state(recorder)).await;
}

pub(super) struct MockLive {
    pub(super) address: SocketAddr,
    keys: Receiver<String>,
    messages: Receiver<Value>,
}

impl MockLive {
    pub(super) fn start() -> io::Result<Self> {
        let ((key_sender, keys), (message_sender, messages)) = (channel(), channel());
        let address = spawned(Channels {
            keys: key_sender,
            messages: message_sender,
        })?;
        Ok(Self {
            address,
            keys,
            messages,
        })
    }

    pub(super) fn heard(self) -> Heard {
        let api_keys = self.keys.try_iter().collect();
        Heard {
            api_keys,
            messages: self.messages.try_iter().collect(),
        }
    }
}
