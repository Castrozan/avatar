use std::sync::Arc;
use std::time::Duration;

use avatar_seat::edges::{HttpProbe, SeatProbe};
use futures_util::{SinkExt, StreamExt};
use std::future::Future;

use serde_json::Value;
use tokio::runtime::Runtime;
use tokio_tungstenite::tungstenite::{Bytes, Message};

use super::connection::{SeatConnection, Socket, open_socket, private_runtime};
use super::hush::hush_the_seat;
use crate::client::{
    HUSH_TIMEOUT_SECONDS, PORT, SPEECH_OPEN_TIMEOUT_SECONDS, SPEECH_PATH, STREAM_PATH, SeatEndpoint,
};
use crate::edge::stage::SystemWaiting;
use crate::failure::Failure;
use crate::waiting::{Clock, wait_until};

pub struct SeatClient {
    runtime: Arc<Runtime>,
    port: u16,
    waiting: SystemWaiting,
}

impl SeatClient {
    pub fn on_port(port: u16) -> Result<Self, Failure> {
        Ok(Self {
            runtime: private_runtime()?,
            port,
            waiting: SystemWaiting::new(),
        })
    }

    pub fn on_this_mac() -> Result<Self, Failure> {
        Self::on_port(PORT)
    }
}

impl Clock for SeatClient {
    fn monotonic(&self) -> f64 {
        self.waiting.monotonic()
    }

    fn sleep(&mut self, seconds: f64) {
        self.waiting.sleep(seconds);
    }
}

async fn play_and_listen(mut socket: Socket, pcm: &[u8]) -> Result<(), Failure> {
    let message = Message::Binary(Bytes::copy_from_slice(pcm));
    let sent = socket.send(message).await;
    sent.map_err(|error| Failure::runtime(&error.to_string()))?;
    while socket.next().await.is_some_and(|frame| frame.is_ok()) {}
    Ok(())
}

async fn play_when_open(
    opening: impl Future<Output = Result<Socket, Failure>>,
    pcm: &[u8],
) -> Result<(), Failure> {
    play_and_listen(opening.await?, pcm).await
}

impl SeatEndpoint for SeatClient {
    type Connection = SeatConnection;

    fn attached(&mut self) -> bool {
        let health = HttpProbe { port: self.port }.health();
        let flag = health.and_then(|body| body.get("attached").and_then(Value::as_bool));
        flag == Some(true)
    }

    fn hush(&mut self) {
        hush_the_seat(self.port, Duration::from_secs(HUSH_TIMEOUT_SECONDS));
    }

    fn play(&mut self, pcm: &[u8]) -> Result<(), Failure> {
        let opening = open_socket(self.port, SPEECH_PATH, SPEECH_OPEN_TIMEOUT_SECONDS);
        self.runtime.block_on(play_when_open(opening, pcm))
    }

    fn wait_until_attached(&mut self, seconds: f64, message: &str) -> Result<(), Failure> {
        wait_until(self, seconds, message, |client| Ok(client.attached()))
    }

    fn connect_stream(&mut self) -> Result<SeatConnection, Failure> {
        let opening = open_socket(self.port, STREAM_PATH, SPEECH_OPEN_TIMEOUT_SECONDS);
        let socket = self.runtime.block_on(opening)?;
        let runtime = Arc::clone(&self.runtime);
        Ok(SeatConnection { runtime, socket })
    }
}
