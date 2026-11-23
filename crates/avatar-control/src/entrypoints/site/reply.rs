use axum::body::Body;
use axum::http::header::{CACHE_CONTROL, CONNECTION, CONTENT_LENGTH, CONTENT_TYPE, LOCATION};
use axum::http::{HeaderName, HeaderValue, StatusCode};
use axum::response::Response;

use super::page::error_page;

pub(super) struct Reply {
    status: StatusCode,
    headers: Vec<(HeaderName, String)>,
    body: Vec<u8>,
}

impl Reply {
    pub(super) fn ok(content_type: &str, body: Vec<u8>) -> Self {
        Self::new(
            StatusCode::OK,
            vec![(CONTENT_TYPE, content_type.to_owned())],
            body,
        )
    }

    fn new(status: StatusCode, headers: Vec<(HeaderName, String)>, body: Vec<u8>) -> Self {
        Self {
            status,
            headers,
            body,
        }
    }

    pub(super) fn with_header(mut self, name: HeaderName, value: String) -> Self {
        self.headers.push((name, value));
        self
    }

    pub(super) fn redirect(location: String) -> Self {
        Self::new(
            StatusCode::MOVED_PERMANENTLY,
            vec![(LOCATION, location)],
            Vec::new(),
        )
    }

    pub(super) fn failure(status: StatusCode, message: &str, explanation: &str) -> Self {
        let page = error_page(status.as_u16(), message, explanation);
        let headers = vec![
            (CONTENT_TYPE, "text/html;charset=utf-8".to_owned()),
            (CONNECTION, "close".to_owned()),
        ];
        Self::new(status, headers, page.into_bytes())
    }

    pub(super) fn into_response(self, with_body: bool) -> Response {
        let length = self.body.len();
        let mut response = Response::new(body_for(self.body, with_body));
        *response.status_mut() = self.status;
        apply_headers(response.headers_mut(), self.headers, length);
        response
    }
}

fn body_for(bytes: Vec<u8>, with_body: bool) -> Body {
    if with_body {
        return Body::from(bytes);
    }
    Body::empty()
}

fn apply_headers(
    headers: &mut axum::http::HeaderMap,
    entries: Vec<(HeaderName, String)>,
    length: usize,
) {
    for (name, value) in entries {
        headers.insert(name, header_value(&value));
    }
    headers.insert(CONTENT_LENGTH, HeaderValue::from(length));
    headers.insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
}

fn header_value(text: &str) -> HeaderValue {
    HeaderValue::from_str(text).unwrap_or_else(|_| HeaderValue::from_static(""))
}
