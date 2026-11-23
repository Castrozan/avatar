use std::error::Error;
use std::net::SocketAddr;
use std::time::Duration;

use avatar_control::camera::FrameSocketServer;
use futures_util::SinkExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::handshake::client::Request;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::http::header::ORIGIN;
use tokio_tungstenite::tungstenite::{Error as WebSocketError, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

use super::super::support::TestResult;

pub(super) const FRAME_BYTE_LENGTH: usize = 16;
pub(super) const ALLOWED_ORIGIN: &str = "http://localhost:3000";
pub(super) const WAIT: Duration = Duration::from_secs(5);

pub(super) type Client = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub(super) struct RunningServer {
    pub(super) address: SocketAddr,
    pub(super) frames: UnboundedReceiver<Vec<u8>>,
}

pub(super) async fn running_server() -> Result<RunningServer, Box<dyn Error>> {
    let server =
        FrameSocketServer::bind(0, FRAME_BYTE_LENGTH, vec![ALLOWED_ORIGIN.to_owned()]).await?;
    let address = server.local_address()?;
    let (sender, frames) = unbounded_channel();
    tokio::spawn(server.serve(sender));
    Ok(RunningServer { address, frames })
}

pub(super) async fn connect(
    address: SocketAddr,
    origin: Option<&'static str>,
) -> Result<Client, WebSocketError> {
    let request = with_origin(format!("ws://{address}/").into_client_request()?, origin);
    Ok(connect_async(request).await?.0)
}

fn with_origin(mut request: Request, origin: Option<&'static str>) -> Request {
    if let Some(origin) = origin {
        request
            .headers_mut()
            .insert(ORIGIN, HeaderValue::from_static(origin));
    }
    request
}

pub(super) async fn accepted_client() -> Result<(RunningServer, Client), Box<dyn Error>> {
    let server = running_server().await?;
    let client = connect(server.address, Some(ALLOWED_ORIGIN)).await?;
    Ok((server, client))
}

pub(super) async fn expect_next(server: &mut RunningServer, expected: Vec<u8>) -> TestResult {
    assert_eq!(next_frame(&mut server.frames).await, Some(expected));
    Ok(())
}

pub(super) async fn assert_refused(origin: Option<&'static str>) -> TestResult {
    let server = running_server().await?;
    let outcome = connect(server.address, origin).await;
    assert!(is_unauthorized(&outcome));
    Ok(())
}

fn is_unauthorized(outcome: &Result<Client, WebSocketError>) -> bool {
    matches!(outcome, Err(WebSocketError::Http(response)) if response.status() == 401)
}

pub(super) async fn send_binary(client: &mut Client, bytes: Vec<u8>) -> TestResult {
    client.send(Message::Binary(bytes.into())).await?;
    Ok(())
}

pub(super) async fn next_frame(frames: &mut UnboundedReceiver<Vec<u8>>) -> Option<Vec<u8>> {
    tokio::time::timeout(WAIT, frames.recv())
        .await
        .ok()
        .flatten()
}

pub(super) fn pattern_frame() -> Vec<u8> {
    (0..FRAME_BYTE_LENGTH)
        .map(|index| u8::try_from(index * 17).unwrap_or_default())
        .collect()
}
