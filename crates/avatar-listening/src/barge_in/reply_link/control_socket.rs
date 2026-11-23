use std::error::Error;
use std::future::pending;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::{Mutex as AsyncMutex, watch};
use tokio::time::sleep;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

mod hearing;

use hearing::hear_until_closed;

use crate::barge_in::reply_link::ReplyLinkState;

pub const CONTROL_SERVER_URL: &str = "ws://127.0.0.1:8765";
const RECONNECT_DELAY: Duration = Duration::from_secs(1);
const IDENTIFICATION: &str = r#"{"type":"identify","role":"agent"}"#;
const FOLLOW_SPEECH: &str = r#"{"type":"followSpeech"}"#;
const STOP_SPEAKING: &str = r#"{"type":"stopSpeaking"}"#;

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;
type Outgoing = SplitSink<Socket, Message>;
type Incoming = SplitStream<Socket>;
type LinkResult<Outcome> = Result<Outcome, Box<dyn Error + Send + Sync>>;

pub struct ReplyLink {
    url: String,
    stopping: watch::Receiver<bool>,
    speaking: Mutex<ReplyLinkState>,
    outgoing: AsyncMutex<Option<Outgoing>>,
}

impl ReplyLink {
    #[must_use]
    pub fn new(url: impl Into<String>, stopping: watch::Receiver<bool>) -> Arc<Self> {
        Arc::new(Self {
            url: url.into(),
            stopping,
            speaking: Mutex::new(ReplyLinkState::default()),
            outgoing: AsyncMutex::new(None),
        })
    }

    #[must_use]
    pub fn speaking(&self) -> bool {
        self.speaking_state().speaking()
    }

    pub async fn stop(&self) {
        let mut outgoing = self.outgoing.lock().await;
        send_when_connected(outgoing.as_mut(), STOP_SPEAKING).await;
    }

    pub async fn follow(&self) {
        follow_until_stopped(self, self.stopping.clone()).await;
    }

    fn speaking_state(&self) -> MutexGuard<'_, ReplyLinkState> {
        self.speaking.lock().unwrap_or_else(PoisonError::into_inner)
    }

    async fn follow_connection(&self) -> LinkResult<()> {
        let stream = self.open_session().await?;
        hear_until_closed(self, stream).await
    }

    async fn open_session(&self) -> LinkResult<Incoming> {
        let (mut sink, stream) = connect_async(&self.url).await?.0.split();
        sink.send(Message::text(IDENTIFICATION)).await?;
        sink.send(Message::text(FOLLOW_SPEECH)).await?;
        *self.outgoing.lock().await = Some(sink);
        Ok(stream)
    }

    async fn lose_connection(&self) {
        *self.outgoing.lock().await = None;
        self.speaking_state().lost_server();
    }
}

async fn send_when_connected(outgoing: Option<&mut Outgoing>, text: &str) {
    if let Some(sink) = outgoing {
        sink.send(Message::text(text)).await.ok();
    }
}

async fn follow_until_stopped(link: &ReplyLink, mut stopping: watch::Receiver<bool>) {
    while !*stopping.borrow() {
        link.follow_connection().await.ok();
        link.lose_connection().await;
        pause_before_reconnect(&mut stopping).await;
    }
}

async fn pause_before_reconnect(stopping: &mut watch::Receiver<bool>) {
    tokio::select! {
        () = sleep(RECONNECT_DELAY) => {}
        () = wait_until_stopped(stopping) => {}
    }
}

async fn wait_until_stopped(stopping: &mut watch::Receiver<bool>) {
    if stopping.wait_for(|stopped| *stopped).await.is_err() {
        pending::<()>().await;
    }
}
