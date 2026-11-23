use std::sync::Arc;
use std::time::Duration;

use futures_util::SinkExt;
use tokio::net::TcpStream;
use tokio::runtime::{Builder, Runtime};
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::handshake::client::Request;
use tokio_tungstenite::tungstenite::http::header::ORIGIN;
use tokio_tungstenite::tungstenite::{Bytes, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

use crate::client::{HOST, PORT, StreamConnection};
use crate::edge::failure::os_failure;
use crate::failure::{Failure, FailureKind};

pub(super) type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

fn handshake_failure(error: &impl ToString) -> Failure {
    Failure::new(FailureKind::Os, &error.to_string())
}

pub(crate) fn private_runtime() -> Result<Arc<Runtime>, Failure> {
    let built = Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build();
    built.map(Arc::new).map_err(|error| os_failure(&error))
}

fn request_for(port: u16, path: &str) -> Result<Request, Failure> {
    let mut request = format!("ws://{HOST}:{port}{path}")
        .into_client_request()
        .map_err(|error| handshake_failure(&error))?;
    let origin = format!("http://{HOST}:{PORT}").parse();
    request
        .headers_mut()
        .insert(ORIGIN, origin.map_err(|error| handshake_failure(&error))?);
    Ok(request)
}

pub(super) async fn open_socket(port: u16, path: &str, seconds: u64) -> Result<Socket, Failure> {
    let request = request_for(port, path)?;
    let opened = timeout(Duration::from_secs(seconds), connect_async(request)).await;
    let opened = opened.map_err(|_| Failure::timeout("timed out during opening handshake"))?;
    opened
        .map(|(socket, _)| socket)
        .map_err(|error| handshake_failure(&error))
}

pub struct SeatConnection {
    pub(super) runtime: Arc<Runtime>,
    pub(super) socket: Socket,
}

async fn send_binary(socket: &mut Socket, data: &[u8]) -> Result<(), Failure> {
    let message = Message::Binary(Bytes::copy_from_slice(data));
    socket
        .send(message)
        .await
        .map_err(|error| handshake_failure(&error))
}

impl StreamConnection for SeatConnection {
    fn send(&mut self, data: &[u8]) -> Result<(), Failure> {
        self.runtime.block_on(send_binary(&mut self.socket, data))
    }

    fn close(&mut self) {
        let closing = self.socket.close(None);
        let _ = self.runtime.block_on(closing);
    }
}
