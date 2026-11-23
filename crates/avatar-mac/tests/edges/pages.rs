use std::error::Error;
use std::net::SocketAddr;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::Message;

use crate::harness::{Page, connect_allowed};

pub(crate) async fn next_message(page: &mut Page) -> Result<Message, Box<dyn Error>> {
    let received = timeout(Duration::from_secs(5), page.next()).await?;
    Ok(received.ok_or("the socket ended")??)
}

fn parsed(reply: &str) -> Result<(u16, String), Box<dyn Error>> {
    let status = reply.split(' ').nth(1).ok_or("no status")?.parse()?;
    let body = reply.split_once("\r\n\r\n").map_or("", |parts| parts.1);
    Ok((status, body.to_owned()))
}

pub(crate) async fn http_get(
    address: SocketAddr,
    path: &str,
    headers: &str,
) -> Result<(u16, String), Box<dyn Error>> {
    let mut stream = TcpStream::connect(address).await?;
    let head = format!("GET {path} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n");
    stream
        .write_all(format!("{head}{headers}\r\n").as_bytes())
        .await?;
    let mut reply = String::new();
    stream.read_to_string(&mut reply).await?;
    parsed(&reply)
}

pub(crate) fn close_of(message: &Message) -> Option<(u16, String)> {
    match message {
        Message::Close(Some(frame)) => Some((frame.code.into(), frame.reason.to_string())),
        _ => None,
    }
}

pub(crate) async fn attached_page(address: SocketAddr) -> Result<Page, Box<dyn Error>> {
    let page = connect_allowed(address, "/audio").await?;
    while !http_get(address, "/health", "").await?.1.contains("true") {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    Ok(page)
}

pub(crate) async fn next_binary(page: &mut Page) -> Result<Vec<u8>, Box<dyn Error>> {
    match next_message(page).await? {
        Message::Binary(bytes) => Ok(bytes.to_vec()),
        other => Err(format!("expected audio, got {other:?}").into()),
    }
}

pub(crate) async fn closed_with(page: &mut Page) -> Result<Option<(u16, String)>, Box<dyn Error>> {
    Ok(close_of(&next_message(page).await?))
}

pub(crate) async fn send_binary(page: &mut Page, bytes: Vec<u8>) -> Result<(), Box<dyn Error>> {
    Ok(page.send(Message::Binary(bytes.into())).await?)
}

pub(crate) async fn announce(page: &mut Page, rate: u32) -> Result<(), Box<dyn Error>> {
    Ok(page
        .send(Message::text(format!("{{\"sampleRate\": {rate}}}")))
        .await?)
}

pub(crate) async fn status_of(
    address: SocketAddr,
    path: &str,
    headers: &str,
) -> Result<u16, Box<dyn Error>> {
    Ok(http_get(address, path, headers).await?.0)
}
