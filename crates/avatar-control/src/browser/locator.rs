use std::time::Duration;

use http_body_util::{BodyExt, Empty};
use hyper::body::Bytes;
use hyper::{Method, Request};
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::TokioExecutor;
use serde_json::Value;

use super::uri_component::encode_uri_component;
use super::{BrowserError, PageTarget, select_page_target};

pub struct TargetLocator {
    base_url: String,
    http: Client<HttpConnector, Empty<Bytes>>,
}

impl TargetLocator {
    #[must_use]
    pub fn new(devtools_port: u16) -> Self {
        Self {
            base_url: format!("http://127.0.0.1:{devtools_port}"),
            http: Client::builder(TokioExecutor::new()).build_http(),
        }
    }

    pub async fn list_targets(&self) -> Result<Vec<PageTarget>, BrowserError> {
        let body = self.request(Method::GET, "/json").await?;
        Ok(serde_json::from_slice(&body)?)
    }

    pub async fn find_page_target(
        &self,
        url_fragment: &str,
    ) -> Result<Option<PageTarget>, BrowserError> {
        let targets = self.list_targets().await?;
        Ok(select_page_target(&targets, url_fragment).cloned())
    }

    pub async fn open_tab(&self, tab_url: &str) -> Result<Value, BrowserError> {
        let route = format!("/json/new?{}", encode_uri_component(tab_url));
        let body = self.request(Method::PUT, &route).await?;
        Ok(serde_json::from_slice(&body)
            .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&body).into_owned())))
    }

    pub async fn find_or_open_page_target(
        &self,
        url_fragment: &str,
        tab_url: &str,
        settle: Duration,
    ) -> Result<Option<PageTarget>, BrowserError> {
        reuse_or_open(self, url_fragment, tab_url, settle).await
    }

    async fn request(&self, method: Method, route_path: &str) -> Result<Bytes, BrowserError> {
        let request = Request::builder()
            .method(method)
            .uri(format!("{}{route_path}", self.base_url))
            .body(Empty::<Bytes>::new())?;
        let response = self.http.request(request).await?;
        Ok(response.into_body().collect().await?.to_bytes())
    }
}

async fn reuse_or_open(
    locator: &TargetLocator,
    url_fragment: &str,
    tab_url: &str,
    settle: Duration,
) -> Result<Option<PageTarget>, BrowserError> {
    if let Some(existing) = locator.find_page_target(url_fragment).await? {
        return Ok(Some(existing));
    }
    locator.open_tab(tab_url).await?;
    tokio::time::sleep(settle).await;
    locator.find_page_target(url_fragment).await
}
