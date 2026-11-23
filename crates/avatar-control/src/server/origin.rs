use axum::http::HeaderMap;
use axum::http::header::ORIGIN;

pub(crate) fn permitted(allowed: &[String], headers: &HeaderMap) -> bool {
    match headers.get(ORIGIN) {
        Some(origin) if !origin.is_empty() => {
            allowed.iter().any(|candidate| origin == candidate.as_str())
        }
        _ => true,
    }
}
