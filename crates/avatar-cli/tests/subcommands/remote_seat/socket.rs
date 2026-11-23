use std::error::Error;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

pub(super) type Socket = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

#[derive(Default)]
pub(super) struct Heard {
    pub(super) levels: String,
    pub(super) voice: usize,
}

pub(super) fn take(heard: &mut Heard, message: Message) {
    match message {
        Message::Text(report) => heard.levels = report.to_string(),
        Message::Binary(chunk) => heard.voice += chunk.len(),
        _ => (),
    }
}

pub(super) async fn open_socket(port: u16) -> Result<Socket, Box<dyn Error>> {
    let mut request = format!("ws://127.0.0.1:{port}/audio").into_client_request()?;
    let origin = format!("http://localhost:{port}").parse()?;
    request.headers_mut().insert("Origin", origin);
    Ok(connect_async(request).await?.0)
}

pub(super) async fn next_message(socket: &mut Socket) -> Result<Message, Box<dyn Error>> {
    let next = tokio::time::timeout(Duration::from_secs(10), socket.next()).await?;
    Ok(next.ok_or("the socket closed")??)
}

pub(super) async fn carry_through_the_audio_socket(port: u16) -> Result<Heard, Box<dyn Error>> {
    let mut socket = open_socket(port).await?;
    socket.send(Message::Binary(vec![7, 8, 9].into())).await?;
    let mut heard = Heard::default();
    while heard.levels.is_empty() || heard.voice == 0 {
        take(&mut heard, next_message(&mut socket).await?);
    }
    Ok(heard)
}
