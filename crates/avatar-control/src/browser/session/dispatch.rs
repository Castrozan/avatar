use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use futures_util::StreamExt;
use serde_json::{Value, json};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};
use tokio::sync::{broadcast, oneshot};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use super::pump::{PendingAnswers, Socket, read_messages, write_messages};

use super::DevToolsSession;
use super::answer::settle_answer;
use crate::browser::BrowserError;

const EVENT_CAPACITY: usize = 256;

pub struct WebSocketDevToolsSession {
    outgoing: UnboundedSender<Message>,
    pending: PendingAnswers,
    events: broadcast::Sender<Value>,
    last_message_id: AtomicU64,
}

impl WebSocketDevToolsSession {
    pub async fn connect(debugger_url: &str) -> Result<Self, BrowserError> {
        let (socket, _response) = connect_async(debugger_url).await?;
        Ok(Self::from_socket(socket))
    }

    fn from_socket(socket: Socket) -> Self {
        let (sink, stream) = socket.split();
        let (outgoing, outgoing_receiver) = unbounded_channel();
        let pending = PendingAnswers::default();
        let events = broadcast::channel(EVENT_CAPACITY).0;
        tokio::spawn(write_messages(sink, outgoing_receiver));
        tokio::spawn(read_messages(stream, Arc::clone(&pending), events.clone()));
        Self::new(outgoing, pending, events)
    }

    fn new(
        outgoing: UnboundedSender<Message>,
        pending: PendingAnswers,
        events: broadcast::Sender<Value>,
    ) -> Self {
        Self {
            outgoing,
            pending,
            events,
            last_message_id: AtomicU64::new(0),
        }
    }

    async fn send_request(
        &self,
        message_id: u64,
        method: &str,
        params: Value,
    ) -> oneshot::Receiver<Value> {
        let (answer_sender, answer_receiver) = oneshot::channel();
        self.pending.lock().await.insert(message_id, answer_sender);
        let payload = json!({ "id": message_id, "method": method, "params": params });
        let _ = self.outgoing.send(Message::text(payload.to_string()));
        answer_receiver
    }

    async fn forget(&self, message_id: u64) {
        self.pending.lock().await.remove(&message_id);
    }
}

impl DevToolsSession for WebSocketDevToolsSession {
    async fn send_and_await_result(
        &self,
        method: &str,
        params: Value,
        timeout: Duration,
    ) -> Result<Value, BrowserError> {
        let message_id = self.last_message_id.fetch_add(1, Ordering::Relaxed) + 1;
        let answer = self.send_request(message_id, method, params).await;
        let received = tokio::time::timeout(timeout, answer)
            .await
            .ok()
            .and_then(Result::ok);
        self.forget(message_id).await;
        settle_answer(received, method)
    }

    fn subscribe_events(&self) -> broadcast::Receiver<Value> {
        self.events.subscribe()
    }

    fn close(&self) {
        let _ = self.outgoing.send(Message::Close(None));
    }
}
