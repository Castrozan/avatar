use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;

use super::fake_edges::FakeLevels;
use super::fakes::Heard;
use super::harness::{Outcome, Running, Socket};

pub(crate) fn parse_reply(text: &str) -> (u16, String) {
    let status = text
        .split(' ')
        .nth(1)
        .and_then(|code| code.parse().ok())
        .unwrap_or(0);
    (
        status,
        text.split("\r\n\r\n").nth(1).unwrap_or("").to_owned(),
    )
}

pub(crate) async fn raw_get(port: u16, path: &str, fetch_site: &str) -> Outcome<String> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).await?;
    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: x\r\nSec-Fetch-Site: {fetch_site}\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(request.as_bytes()).await?;
    let mut reply = Vec::new();
    stream.read_to_end(&mut reply).await?;
    Ok(String::from_utf8_lossy(&reply).into_owned())
}

impl Running {
    pub(crate) fn new(
        port: u16,
        fakes: (Heard, FakeLevels),
        handles: (oneshot::Sender<()>, JoinHandle<()>),
        directory: tempfile::TempDir,
    ) -> Self {
        let (heard, levels) = fakes;
        Self {
            port,
            heard,
            levels,
            stop: Some(handles.0),
            serving: handles.1,
            _directory: directory,
        }
    }

    pub(crate) async fn fetch(&self, path: &str, fetch_site: &str) -> Outcome<(u16, String)> {
        Ok(parse_reply(&raw_get(self.port, path, fetch_site).await?))
    }

    pub(crate) async fn get(&self, path: &str) -> Outcome<(u16, String)> {
        self.fetch(path, "same-origin").await
    }

    pub(crate) async fn health(&self) -> Outcome<Value> {
        Ok(serde_json::from_str(
            &self.fetch("/health", "same-origin").await?.1,
        )?)
    }

    pub(crate) async fn socket(&self, path: &str, origin: Option<&str>) -> Outcome<Socket> {
        let mut request = format!("ws://127.0.0.1:{}{path}", self.port).into_client_request()?;
        let own = format!("http://localhost:{}", self.port);
        request
            .headers_mut()
            .insert("Origin", HeaderValue::from_str(origin.unwrap_or(&own))?);
        Ok(connect_async(request).await.map(|(socket, _)| socket)?)
    }

    pub(crate) fn stop(&mut self) {
        let _ = self.stop.take().map(|stop| stop.send(()));
    }
}
