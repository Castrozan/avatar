use axum::extract::ws::{Message, Utf8Bytes, WebSocket};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::server::command::Command;
use crate::server::loop_support::KeepAlive;
use crate::session::ClientId;

pub(crate) type Incoming = Option<Result<Message, axum::Error>>;

pub(crate) enum Event {
    Incoming(Incoming),
    Outgoing(Option<String>),
    Beat,
}

pub(crate) async fn next_event(
    websocket: &mut WebSocket,
    outbound: &mut UnboundedReceiver<String>,
    keep_alive: &mut KeepAlive,
) -> Event {
    tokio::select! {
        event = next_socket_or_outbound(websocket, outbound) => event,
        () = keep_alive.tick() => Event::Beat,
    }
}

async fn next_socket_or_outbound(
    websocket: &mut WebSocket,
    outbound: &mut UnboundedReceiver<String>,
) -> Event {
    tokio::select! {
        incoming = websocket.recv() => Event::Incoming(incoming),
        text = outbound.recv() => Event::Outgoing(text),
    }
}

pub(crate) fn frame(client: ClientId, text: &Utf8Bytes) -> Command {
    Command::Frame {
        client,
        text: text.to_string(),
    }
}

pub(crate) async fn write(websocket: &mut WebSocket, text: Option<String>) -> bool {
    match text {
        Some(text) => websocket.send(Message::Text(text.into())).await.is_ok(),
        None => close(websocket).await,
    }
}

async fn close(websocket: &mut WebSocket) -> bool {
    let _ = websocket.send(Message::Close(None)).await;
    false
}
