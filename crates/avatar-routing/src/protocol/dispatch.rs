use serde_json::{Value, json};

use crate::names::vocabulary::Vocabulary;
use crate::protocol::operations::perform;
use crate::protocol::request::Request;

#[must_use]
pub fn respond(line: &str, default_vocabulary: Option<&Vocabulary>) -> Value {
    match serde_json::from_str::<Request>(line) {
        Ok(request) => {
            let vocabulary = request.vocabulary.as_ref().or(default_vocabulary);
            json!({"ok": perform(request.operation, vocabulary)})
        }
        Err(error) => json!({"error": error.to_string()}),
    }
}
