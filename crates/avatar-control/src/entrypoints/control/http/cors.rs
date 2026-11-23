use axum::extract::{Request, State};
use axum::http::header::{HeaderName, HeaderValue, ORIGIN, VARY};
use axum::middleware::Next;
use axum::response::Response;

use super::HttpState;

const ALLOW_ORIGIN: HeaderName = HeaderName::from_static("access-control-allow-origin");
const ALLOW_METHODS: HeaderName = HeaderName::from_static("access-control-allow-methods");
const ALLOW_HEADERS: HeaderName = HeaderName::from_static("access-control-allow-headers");

pub(super) async fn allow_origins(
    State(state): State<HttpState>,
    request: Request,
    next: Next,
) -> Response {
    let granted = granted_origin(&state, &request);
    let mut response = next.run(request).await;
    apply_headers(&mut response, granted);
    response
}

fn granted_origin(state: &HttpState, request: &Request) -> Option<HeaderValue> {
    let origin = request.headers().get(ORIGIN)?;
    let text = origin.to_str().ok()?;
    state
        .origins
        .iter()
        .any(|allowed| allowed == text)
        .then(|| origin.clone())
}

fn apply_headers(response: &mut Response, granted: Option<HeaderValue>) {
    let headers = response.headers_mut();
    if let Some(origin) = granted {
        headers.insert(ALLOW_ORIGIN, origin);
    }
    headers.insert(VARY, HeaderValue::from_static("Origin"));
    headers.insert(ALLOW_METHODS, HeaderValue::from_static("GET"));
    headers.insert(ALLOW_HEADERS, HeaderValue::from_static("Content-Type"));
}
