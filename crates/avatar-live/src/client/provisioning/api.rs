use reqwest::{Client, Method, RequestBuilder, Response};
use serde_json::Value;

use crate::client::ClientError;

const KEY_HEADER: &str = "xi-api-key";

#[derive(Clone)]
pub struct AgentApi {
    base: String,
    key: String,
    http: Client,
}

fn transport(failure: reqwest::Error) -> ClientError {
    ClientError(failure.without_url().to_string())
}

fn with_body(request: RequestBuilder, body: Option<&Value>) -> RequestBuilder {
    match body {
        Some(json) => request
            .header("content-type", "application/json")
            .body(json.to_string()),
        None => request,
    }
}

async fn parsed(method: &Method, path: &str, response: Response) -> Result<Value, ClientError> {
    let status = response.status();
    let text = response.text().await.map_err(transport)?;
    if !status.is_success() {
        return Err(ClientError(format!("{method} {path}: {status} {text}")));
    }
    serde_json::from_str(&text).map_err(|failure| ClientError(failure.to_string()))
}

impl AgentApi {
    #[must_use]
    pub fn new(base: &str, key: &str) -> Self {
        Self {
            base: base.trim_end_matches('/').to_owned(),
            key: key.trim().to_owned(),
            http: Client::new(),
        }
    }

    pub(super) async fn get(&self, path: &str) -> Result<Value, ClientError> {
        self.call(Method::GET, path, None).await
    }

    pub(super) async fn post(&self, path: &str, body: &Value) -> Result<Value, ClientError> {
        self.call(Method::POST, path, Some(body)).await
    }

    pub(super) async fn patch(&self, path: &str, body: &Value) -> Result<Value, ClientError> {
        self.call(Method::PATCH, path, Some(body)).await
    }

    async fn call(
        &self,
        method: Method,
        path: &str,
        body: Option<&Value>,
    ) -> Result<Value, ClientError> {
        let request = self
            .http
            .request(method.clone(), format!("{}{path}", self.base));
        let request = with_body(request.header(KEY_HEADER, &self.key), body);
        let response = request.send().await.map_err(transport)?;
        parsed(&method, path, response).await
    }
}
