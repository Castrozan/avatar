use std::collections::HashMap;
use std::sync::Arc;

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::net::TcpStream;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::{Mutex, broadcast, oneshot};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub(super) type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;
pub(super) type PendingAnswers = Arc<Mutex<HashMap<u64, oneshot::Sender<Value>>>>;

pub(super) async fn write_messages(
    mut sink: SplitSink<Socket, Message>,
    mut outgoing: UnboundedReceiver<Message>,
) {
    while let Some(message) = outgoing.recv().await
        && sink.send(message).await.is_ok()
    {}
}

pub(super) async fn read_messages(
    mut stream: SplitStream<Socket>,
    pending: PendingAnswers,
    events: broadcast::Sender<Value>,
) {
    while let Some(Ok(frame)) = stream.next().await {
        dispatch_frame(frame, &pending, &events).await;
    }
    pending.lock().await.clear();
}

async fn dispatch_frame(
    frame: Message,
    pending: &PendingAnswers,
    events: &broadcast::Sender<Value>,
) {
    let Message::Text(text) = frame else {
        return;
    };
    dispatch_text(text.as_str(), pending, events).await;
}

async fn dispatch_text(text: &str, pending: &PendingAnswers, events: &broadcast::Sender<Value>) {
    let Ok(message) = serde_json::from_str::<Value>(text) else {
        return;
    };
    resolve_answer(&message, pending).await;
    let _ = events.send(message);
}

async fn resolve_answer(message: &Value, pending: &PendingAnswers) {
    let Some(message_id) = message.get("id").and_then(Value::as_u64) else {
        return;
    };
    let sender = pending.lock().await.remove(&message_id);
    let _ = sender.map(|answer| answer.send(message.clone()));
}
