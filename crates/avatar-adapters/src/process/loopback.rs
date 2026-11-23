use std::time::Duration;

use reqwest::blocking::{Client, Response};

const LOOPBACK_HOST: &str = "127.0.0.1";

#[must_use]
pub fn loopback_get(port: u16, path: &str, timeout: Duration) -> Option<Response> {
    let client = Client::builder().timeout(timeout).build().ok()?;
    client
        .get(format!("http://{LOOPBACK_HOST}:{port}{path}"))
        .send()
        .ok()
}
