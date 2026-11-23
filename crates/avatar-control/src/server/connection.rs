use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use axum::body::Bytes;
use axum::extract::State;
use axum::extract::ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use tokio::sync::mpsc::{Sender, UnboundedReceiver, unbounded_channel};

use crate::server::command::Command;
use crate::server::event::{Event, Incoming, frame, next_event, write};
use crate::server::loop_support::KeepAlive;
use crate::server::origin;
use crate::session::ClientId;

#[derive(Clone)]
pub(crate) struct ConnectionState {
    pub(crate) commands: Sender<Command>,
    pub(crate) client_ids: Arc<AtomicU64>,
    pub(crate) origins: Arc<[String]>,
}

pub(crate) async fn upgrade(
    State(state): State<ConnectionState>,
    headers: HeaderMap,
    socket: WebSocketUpgrade,
) -> Response {
    if !origin::permitted(&state.origins, &headers) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let client = ClientId(state.client_ids.fetch_add(1, Ordering::Relaxed));
    socket.on_upgrade(move |websocket| drive(Connection::new(websocket, client, state.commands)))
}

struct Connection {
    websocket: WebSocket,
    client: ClientId,
    commands: Sender<Command>,
    outbound: UnboundedReceiver<String>,
    keep_alive: KeepAlive,
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
            keep_alive: KeepAlive::new(),
        }
    }

    async fn step(&mut self) -> bool {
        let event = next_event(
            &mut self.websocket,
            &mut self.outbound,
            &mut self.keep_alive,
        )
        .await;
        self.handle(event).await
    }

    async fn handle(&mut self, event: Event) -> bool {
        match event {
            Event::Incoming(incoming) => self.relay(incoming).await,
            Event::Outgoing(text) => write(&mut self.websocket, text).await,
            Event::Beat => self.beat().await,
        }
    }

    async fn relay(&mut self, incoming: Incoming) -> bool {
        match incoming {
            Some(Ok(Message::Text(text))) => self.forward(&text).await,
            Some(Ok(Message::Pong(_))) => self.acknowledge(),
            Some(Ok(Message::Close(_)) | Err(_)) | None => false,
            Some(Ok(_)) => true,
        }
    }

    async fn forward(&mut self, text: &Utf8Bytes) -> bool {
        self.commands.send(frame(self.client, text)).await.is_ok()
    }

    fn acknowledge(&mut self) -> bool {
        self.keep_alive.record_pong();
        true
    }

    async fn beat(&mut self) -> bool {
        self.keep_alive.begin_probe()
            && self
                .websocket
                .send(Message::Ping(Bytes::default()))
                .await
                .is_ok()
    }
}

async fn drive(mut connection: Connection) {
    while connection.step().await {}
    let _ = connection
        .commands
        .send(Command::Disconnect(connection.client))
        .await;
}
