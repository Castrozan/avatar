use std::future::{Future, ready};
use std::sync::{Arc, Mutex, PoisonError};

use avatar_transcription::error::Outcome;
use avatar_transcription::{HttpRequest, HttpResponse, HttpSender};
use serde_json::{Value, json};

pub(super) const SCRIBE_TEXT: &str = "hello from scribe";
pub(super) const HEX_TEXT: &str = "hello from hex";

#[derive(Clone)]
pub(super) struct ScriptedSender {
    scribe_status: u16,
    requests: Arc<Mutex<Vec<HttpRequest>>>,
}

impl ScriptedSender {
    pub(super) fn answering_scribe_with(scribe_status: u16) -> Self {
        Self {
            scribe_status,
            requests: Arc::default(),
        }
    }

    pub(super) fn requests(&self) -> Vec<HttpRequest> {
        let requests = self.requests.lock().unwrap_or_else(PoisonError::into_inner);
        requests.clone()
    }

    fn answer_to(&self, request: &HttpRequest) -> HttpResponse {
        let url = request.url.as_str();
        match url {
            _ if url.starts_with("https://") => response(self.scribe_status, &scribe_answer()),
            _ if url.ends_with("/models") => response(200, &hex_models()),
            _ => response(200, &hex_answer()),
        }
    }
}

fn scribe_answer() -> Value {
    json!({"text": SCRIBE_TEXT, "language_code": "eng", "language_probability": 0.99})
}

fn hex_models() -> Value {
    json!([{"id": "parakeet_v3", "installed": true}])
}

fn hex_answer() -> Value {
    json!({"transcript": HEX_TEXT})
}

fn response(status: u16, body: &Value) -> HttpResponse {
    HttpResponse {
        status,
        reason: "Scripted".to_owned(),
        body: body.to_string().into_bytes(),
    }
}

impl HttpSender for ScriptedSender {
    fn send(&self, request: HttpRequest) -> impl Future<Output = Outcome<HttpResponse>> + Send {
        let answer = self.answer_to(&request);
        let mut requests = self.requests.lock().unwrap_or_else(PoisonError::into_inner);
        requests.push(request);
        ready(Ok(answer))
    }
}
