use axum::Router;
use axum::body::{Body, to_bytes};
use axum::extract::{Request, State};
use axum::http::HeaderMap;
use axum::http::request::Parts;
use axum::response::Response;
use serde_json::Value;
use std::collections::VecDeque;
use std::error::Error;
use std::future::IntoFuture;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RecordedRequest {
    pub(crate) method: String,
    pub(crate) target: String,
    pub(crate) headers: Vec<(String, String)>,
    pub(crate) body: Vec<u8>,
}

#[derive(Debug, Default)]
struct Script {
    responses: VecDeque<(u16, Vec<u8>)>,
    requests: Vec<RecordedRequest>,
}

impl Script {
    fn new(responses: Vec<(u16, Value)>) -> Self {
        let responses = responses
            .into_iter()
            .map(|(status, payload)| (status, payload.to_string().into_bytes()));
        Self {
            responses: responses.collect(),
            requests: Vec::new(),
        }
    }
}

pub(crate) struct MockServer {
    pub(crate) port: u16,
    script: Arc<Mutex<Script>>,
}

impl MockServer {
    pub(crate) async fn start(responses: Vec<(u16, Value)>) -> Result<Self, Box<dyn Error>> {
        let script = Arc::new(Mutex::new(Script::new(responses)));
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        let router = Router::new()
            .fallback(handle)
            .with_state(Arc::clone(&script));
        tokio::spawn(axum::serve(listener, router).into_future());
        Ok(Self { port, script })
    }

    pub(crate) async fn recorded(&self) -> Vec<RecordedRequest> {
        self.script.lock().await.requests.clone()
    }

    pub(crate) async fn remaining(&self) -> usize {
        self.script.lock().await.responses.len()
    }
}

async fn handle(State(script): State<Arc<Mutex<Script>>>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let body = to_bytes(body, usize::MAX).await.unwrap_or_default();
    let mut guard = script.lock().await;
    guard.requests.push(recorded_request(&parts, &body));
    let (status, payload) = guard.responses.pop_front().unwrap_or((500, b"{}".to_vec()));
    json_response(status, payload)
}

fn recorded_request(parts: &Parts, body: &[u8]) -> RecordedRequest {
    RecordedRequest {
        method: parts.method.to_string(),
        target: parts
            .uri
            .path_and_query()
            .map_or_else(String::new, ToString::to_string),
        headers: header_pairs(&parts.headers),
        body: body.to_vec(),
    }
}

fn header_pairs(headers: &HeaderMap) -> Vec<(String, String)> {
    headers
        .iter()
        .map(|(name, value)| {
            (
                name.as_str().to_owned(),
                value.to_str().unwrap_or_default().to_owned(),
            )
        })
        .collect()
}

fn json_response(status: u16, payload: Vec<u8>) -> Response {
    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Body::from(payload))
        .unwrap_or_default()
}
