use std::io;
use std::sync::Arc;
use std::time::Duration;

use avatar_room_audio::mouth::VoiceLevelSink;

use crate::edge::client::private_runtime;
use futures_util::SinkExt;
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

pub const CONTROL_URL: &str = "ws://127.0.0.1:8765";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const IDENTIFY: &str = "{\"type\": \"identify\", \"role\": \"agent\"}";

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub struct ControlLevels {
    runtime: Arc<Runtime>,
    socket: Socket,
}

fn failure(message: &impl ToString) -> io::Error {
    io::Error::other(message.to_string())
}

fn opened(runtime: &Runtime, url: &str) -> io::Result<Socket> {
    let opening = async { timeout(CONNECT_TIMEOUT, connect_async(url)).await };
    let (mut socket, _) = runtime
        .block_on(opening)
        .map_err(|error| failure(&error))?
        .map_err(|error| failure(&error))?;
    let identified = runtime.block_on(socket.send(Message::text(IDENTIFY)));
    identified.map_err(|error| failure(&error)).map(|()| socket)
}

impl ControlLevels {
    pub fn connect(url: &str) -> io::Result<Self> {
        let runtime = private_runtime().map_err(|error| failure(&error.message))?;
        let socket = opened(&runtime, url)?;
        Ok(Self { runtime, socket })
    }
}

impl VoiceLevelSink for ControlLevels {
    fn send_text(&mut self, text: &str) -> io::Result<()> {
        let sending = self.socket.send(Message::text(text));
        self.runtime
            .block_on(sending)
            .map_err(|error| failure(&error))
    }

    fn close(&mut self) {
        let _ = self.runtime.block_on(self.socket.close(None));
    }
}

#[cfg(test)]
mod tests;
