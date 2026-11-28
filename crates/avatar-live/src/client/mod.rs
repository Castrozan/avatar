mod frames;
mod handshake;
pub mod provisioning;

use std::fmt;

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Error as SocketError;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::events::LiveEvent;
use crate::session::{audio_chunk, delivery};
use crate::translation::EventTranslator;

pub use handshake::SessionRequest;

pub(super) type Outcome<Value> = Result<Value, ClientError>;

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Debug)]
pub struct ClientError(pub String);

impl fmt::Display for ClientError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for ClientError {}

impl From<SocketError> for ClientError {
    fn from(failure: SocketError) -> Self {
        Self(failure.to_string())
    }
}

pub struct LiveClient {
    pub(super) sink: SplitSink<Socket, Message>,
    pub(super) stream: SplitStream<Socket>,
    pub(super) translator: EventTranslator,
    pub(super) ended: bool,
}

fn tolerated(failure: ClientError, ended: bool) -> Result<(), ClientError> {
    if ended {
        return Ok(());
    }
    Err(failure)
}

impl LiveClient {
    pub(super) fn new(socket: Socket) -> Self {
        let (sink, stream) = socket.split();
        Self {
            sink,
            stream,
            translator: EventTranslator::default(),
            ended: false,
        }
    }

    pub async fn send_audio(&mut self, pcm: &[u8]) -> Result<(), ClientError> {
        self.send(&audio_chunk(pcm)).await
    }

    pub async fn deliver(&mut self, result: &str, is_final: bool) -> Result<(), ClientError> {
        self.send(&delivery(result, is_final)).await
    }

    pub async fn next_events(&mut self) -> Result<Option<Vec<LiveEvent>>, ClientError> {
        frames::next_events(self).await
    }

    pub async fn close(&mut self) -> Result<(), ClientError> {
        let closed = self.sink.close().await.map_err(ClientError::from);
        closed.or_else(|failure| tolerated(failure, self.ended))
    }

    pub(super) async fn send(&mut self, value: &Value) -> Result<(), ClientError> {
        self.sink
            .send(Message::text(value.to_string()))
            .await
            .map_err(ClientError::from)
    }
}
