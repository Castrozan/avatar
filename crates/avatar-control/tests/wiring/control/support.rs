use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::fs;
use std::net::SocketAddr;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

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

pub(crate) fn fixture_cases(text: &str, key: &str) -> Result<Vec<Value>, Box<dyn Error>> {
    let fixture: Value = serde_json::from_str(text)?;
    Ok(fixture[key].as_array().cloned().unwrap_or_default())
}

pub(crate) fn fake_executable(
    directory: &Path,
    name: &str,
    body: &str,
) -> Result<PathBuf, Box<dyn Error>> {
    let path = directory.join(name);
    fs::write(&path, format!("#!/bin/sh\n{body}\n"))?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755))?;
    Ok(path)
}

pub(crate) struct HttpAnswer {
    pub(crate) status: u16,
    pub(crate) headers: HashMap<String, String>,
    pub(crate) body: String,
}

impl HttpAnswer {
    pub(crate) fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name).map(String::as_str)
    }
}

pub(crate) async fn http_request(
    address: SocketAddr,
    method: &str,
    path: &str,
    origin: Option<&str>,
) -> Result<HttpAnswer, Box<dyn Error>> {
    let mut stream = TcpStream::connect(address).await?;
    stream
        .write_all(request_text(method, path, origin).as_bytes())
        .await?;
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).await?;
    parse_answer(&String::from_utf8_lossy(&raw))
}

fn request_text(method: &str, path: &str, origin: Option<&str>) -> String {
    let origin_line = origin.map_or(String::new(), |value| format!("Origin: {value}\r\n"));
    format!("{method} {path} HTTP/1.1\r\nHost: localhost\r\n{origin_line}Connection: close\r\n\r\n")
}

fn parse_answer(text: &str) -> Result<HttpAnswer, Box<dyn Error>> {
    let (head, body) = text.split_once("\r\n\r\n").ok_or("no header terminator")?;
    Ok(HttpAnswer {
        status: status_of(head)?,
        headers: headers_of(head),
        body: body.to_owned(),
    })
}

fn status_of(head: &str) -> Result<u16, Box<dyn Error>> {
    let line = head.lines().next().ok_or("no status line")?;
    Ok(line.split(' ').nth(1).ok_or("no status code")?.parse()?)
}

fn headers_of(head: &str) -> HashMap<String, String> {
    head.lines()
        .filter_map(|line| line.split_once(": "))
        .map(|(name, value)| (name.to_ascii_lowercase(), value.to_owned()))
        .collect()
}
