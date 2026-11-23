use std::time::Duration;

use http_body_util::{BodyExt, Empty};
use hyper::body::Bytes;
use hyper::{Method, Request};
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use serde_json::Value;
use tokio::time::timeout;

pub async fn seat_is_attached(health_url: &str, patience: Duration) -> bool {
    matches!(
        timeout(patience, fetch_attached(health_url)).await,
        Ok(Some(true))
    )
}

async fn fetch_attached(health_url: &str) -> Option<bool> {
    let body = fetch_body(health_url).await?;
    let health: Value = serde_json::from_slice(&body).ok()?;
    Some(health.get("attached") == Some(&Value::Bool(true)))
}

async fn fetch_body(health_url: &str) -> Option<Bytes> {
    let client = Client::builder(TokioExecutor::new()).build_http();
    let request = Request::builder()
        .method(Method::GET)
        .uri(health_url)
        .body(Empty::<Bytes>::new())
        .ok()?;
    let response = client.request(request).await.ok()?;
    Some(response.into_body().collect().await.ok()?.to_bytes())
}
