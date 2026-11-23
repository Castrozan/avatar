use crate::error::{Outcome, transport_failure};
use std::future::Future;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
    pub timeout: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub reason: String,
    pub body: Vec<u8>,
}

pub trait HttpSender: Sync {
    fn send(&self, request: HttpRequest) -> impl Future<Output = Outcome<HttpResponse>> + Send;
}

#[derive(Debug, Clone, Default)]
pub struct ReqwestSender {
    client: reqwest::Client,
}

impl HttpSender for ReqwestSender {
    fn send(&self, request: HttpRequest) -> impl Future<Output = Outcome<HttpResponse>> + Send {
        execute(self.client.clone(), request)
    }
}

async fn execute(client: reqwest::Client, request: HttpRequest) -> Outcome<HttpResponse> {
    let builder = with_body(with_headers(&client, &request), &request);
    let response = builder
        .send()
        .await
        .map_err(|error| transport_failure(&error))?;
    read_response(response).await
}

async fn read_response(response: reqwest::Response) -> Outcome<HttpResponse> {
    let status = response.status();
    let body = response
        .bytes()
        .await
        .map_err(|error| transport_failure(&error))?;
    Ok(response_of(status, body.to_vec()))
}

fn response_of(status: reqwest::StatusCode, body: Vec<u8>) -> HttpResponse {
    let reason = status.canonical_reason().unwrap_or_default().to_owned();
    HttpResponse {
        status: status.as_u16(),
        reason,
        body,
    }
}

fn with_headers(client: &reqwest::Client, request: &HttpRequest) -> reqwest::RequestBuilder {
    let method = match request.method {
        HttpMethod::Get => reqwest::Method::GET,
        HttpMethod::Post => reqwest::Method::POST,
    };
    request.headers.iter().fold(
        client
            .request(method, &request.url)
            .timeout(request.timeout),
        |builder, (name, value)| builder.header(name, value),
    )
}

fn with_body(builder: reqwest::RequestBuilder, request: &HttpRequest) -> reqwest::RequestBuilder {
    match &request.body {
        Some(body) => builder.body(body.clone()),
        None => builder,
    }
}
