mod answer;
mod keyterms;
mod language_flow;
mod recent_language;
mod request;

use crate::error::TranscriptionError;
use crate::http::{HttpMethod, HttpRequest, HttpResponse, HttpSender};
use crate::segment::wav_bytes;
use recent_language::RecentLanguage;
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

pub use keyterms::{KEYTERMS_BEFORE_MINIMUM_BILLING, KeytermSource, RECOGNIZED_TERMS, keyterms};
pub use request::{SCRIBE_MODEL, multipart_body, scribe_fields};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

fn decoded_answer(response: HttpResponse) -> Result<Value, TranscriptionError> {
    if response.status >= 400 {
        return Err(TranscriptionError::Status {
            code: response.status,
            reason: response.reason,
        });
    }
    serde_json::from_slice(&response.body)
        .map_err(|error| TranscriptionError::MalformedBody(error.to_string()))
}

#[derive(Debug)]
pub struct ScribeClient<S> {
    sender: S,
    endpoint: String,
    api_key: String,
    keyterm_source: KeytermSource,
    recent_language: RecentLanguage,
    boundary_sequence: AtomicU64,
}

impl<S: HttpSender> ScribeClient<S> {
    pub fn new(
        sender: S,
        endpoint: String,
        api_key: String,
        keyterm_source: KeytermSource,
    ) -> Self {
        Self {
            sender,
            endpoint,
            api_key,
            keyterm_source,
            recent_language: RecentLanguage::default(),
            boundary_sequence: AtomicU64::new(0),
        }
    }

    #[must_use]
    pub fn keyterms(&self) -> &[String] {
        self.keyterm_source.terms()
    }

    pub(super) async fn answer(
        &self,
        pcm: &[u8],
        language_code: Option<&str>,
    ) -> Result<Value, TranscriptionError> {
        let boundary = format!(
            "avatar-boundary-{:016x}",
            self.boundary_sequence.fetch_add(1, Ordering::Relaxed)
        );
        let request = self.multipart_request(pcm, language_code, &boundary);
        let response = self.sender.send(request).await?;
        decoded_answer(response)
    }

    fn multipart_request(
        &self,
        pcm: &[u8],
        language_code: Option<&str>,
        boundary: &str,
    ) -> HttpRequest {
        let fields = scribe_fields(language_code, self.keyterm_source.terms());
        let body = multipart_body(boundary, &fields, &wav_bytes(pcm));
        self.post(boundary, body)
    }

    fn post(&self, boundary: &str, body: Vec<u8>) -> HttpRequest {
        HttpRequest {
            method: HttpMethod::Post,
            url: self.endpoint.clone(),
            headers: self.headers(boundary),
            body: Some(body),
            timeout: REQUEST_TIMEOUT,
        }
    }

    fn headers(&self, boundary: &str) -> Vec<(String, String)> {
        vec![
            ("xi-api-key".to_owned(), self.api_key.clone()),
            (
                "Content-Type".to_owned(),
                format!("multipart/form-data; boundary={boundary}"),
            ),
        ]
    }
}
