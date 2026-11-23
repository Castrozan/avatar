use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::net::SocketAddr;

use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub(crate) type TestResult = Result<(), Box<dyn Error>>;

pub(crate) fn check(condition: bool, detail: impl Display) -> TestResult {
    if condition {
        return Ok(());
    }
    Err(detail.to_string().into())
}

pub(crate) struct Answer {
    pub(crate) status: u64,
    pub(crate) headers: HashMap<String, String>,
    pub(crate) body: Vec<u8>,
}

fn header_line(name: &str, value: &Value) -> String {
    format!("{name}: {}\r\n", value.as_str().unwrap_or_default())
}

fn request_text(method: &str, path: &str, headers: &Value) -> String {
    let mut text = format!("{method} {path} HTTP/1.1\r\nHost: localhost\r\n");
    for (name, value) in headers.as_object().into_iter().flatten() {
        text.push_str(&header_line(name, value));
    }
    text.push_str("Connection: close\r\n\r\n");
    text
}

fn header_map(head: &str) -> HashMap<String, String> {
    head.lines()
        .skip(1)
        .filter_map(|line| line.split_once(": "))
        .map(|(name, value)| (name.to_ascii_lowercase(), value.to_owned()))
        .collect()
}

fn head_end(raw: &[u8]) -> Result<usize, Box<dyn Error>> {
    Ok(raw
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or("no head")?)
}

fn parse(raw: &[u8]) -> Result<Answer, Box<dyn Error>> {
    let split = head_end(raw)?;
    let head = String::from_utf8_lossy(raw.get(..split).ok_or("head")?).into_owned();
    let status = head.split(' ').nth(1).ok_or("no status")?.parse()?;
    let body = raw.get(split + 4..).unwrap_or_default().to_vec();
    Ok(Answer {
        status,
        headers: header_map(&head),
        body,
    })
}

pub(crate) async fn fetch(
    address: SocketAddr,
    method: &str,
    path: &str,
    headers: &Value,
) -> Result<Answer, Box<dyn Error>> {
    let mut stream = TcpStream::connect(address).await?;
    stream
        .write_all(request_text(method, path, headers).as_bytes())
        .await?;
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).await?;
    parse(&raw)
}
