use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use axum::extract::State;
use axum::extract::ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade};
use axum::response::Response;
use tokio::sync::mpsc::{Sender, UnboundedReceiver, unbounded_channel};

use crate::server::command::Command;
use crate::session::ClientId;

type Incoming = Option<Result<Message, axum::Error>>;

#[derive(Clone)]
pub(crate) struct ConnectionState {
    pub(crate) commands: Sender<Command>,
    pub(crate) client_ids: Arc<AtomicU64>,
}

pub(crate) async fn upgrade(
    State(state): State<ConnectionState>,
    socket: WebSocketUpgrade,
) -> Response {
    let client = ClientId(state.client_ids.fetch_add(1, Ordering::Relaxed));
    socket.on_upgrade(move |websocket| drive(Connection::new(websocket, client, state.commands)))
}

struct Connection {
    websocket: WebSocket,
    client: ClientId,
    commands: Sender<Command>,
    outbound: UnboundedReceiver<String>,
}

impl Connection {
    fn new(websocket: WebSocket, client: ClientId, commands: Sender<Command>) -> Self {
        let (sender, outbound) = unbounded_channel();
        let _ = commands.try_send(Command::Connect { client, sender });
        Self {
            websocket,
            client,
            commands,
            outbound,
        }
    }

    async fn step(&mut self) -> bool {
        let event = next_event(&mut self.websocket, &mut self.outbound).await;
        match event {
            Event::Incoming(incoming) => relay(&self.commands, self.client, incoming).await,
            Event::Outgoing(text) => write(&mut self.websocket, text).await,
        }
    }
}

enum Event {
    Incoming(Incoming),
    Outgoing(Option<String>),
}

async fn next_event(websocket: &mut WebSocket, outbound: &mut UnboundedReceiver<String>) -> Event {
    tokio::select! {
        incoming = websocket.recv() => Event::Incoming(incoming),
        text = outbound.recv() => Event::Outgoing(text),
    }
}

async fn drive(mut connection: Connection) {
    while connection.step().await {}
    let _ = connection
        .commands
        .send(Command::Disconnect(connection.client))
        .await;
}

async fn relay(commands: &Sender<Command>, client: ClientId, incoming: Incoming) -> bool {
    match incoming {
        Some(Ok(Message::Text(text))) => commands.send(frame(client, &text)).await.is_ok(),
        Some(Ok(Message::Close(_)) | Err(_)) | None => false,
        Some(Ok(_)) => true,
    }
}

fn frame(client: ClientId, text: &Utf8Bytes) -> Command {
    Command::Frame {
        client,
        text: text.to_string(),
    }
}

async fn write(websocket: &mut WebSocket, text: Option<String>) -> bool {
    match text {
        Some(text) => websocket.send(Message::Text(text.into())).await.is_ok(),
        None => false,
    }
}
