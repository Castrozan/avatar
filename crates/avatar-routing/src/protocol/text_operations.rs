use serde::Deserialize;
use serde_json::{Value, json};

use crate::hearing::wake_phrase::opens_with_greeting;
use crate::text::folding::{folded, word_matches};
use crate::text::similarity::ratio;
use crate::text::sound::sound_key;

#[derive(Deserialize)]
pub struct FoldedRequest {
    pub text: String,
}

impl FoldedRequest {
    #[must_use]
    pub fn answer(&self) -> Value {
        json!(folded(&self.text))
    }
}

#[derive(Deserialize)]
pub struct SoundKeyRequest {
    pub word: String,
}

impl SoundKeyRequest {
    #[must_use]
    pub fn answer(&self) -> Value {
        json!(sound_key(&self.word))
    }
}

#[derive(Deserialize)]
pub struct RatioRequest {
    pub left: String,
    pub right: String,
}

impl RatioRequest {
    #[must_use]
    pub fn answer(&self) -> Value {
        json!(ratio(&self.left, &self.right))
    }
}

#[derive(Deserialize)]
pub struct WordMatchesRequest {
    pub text: String,
}

impl WordMatchesRequest {
    #[must_use]
    pub fn answer(&self) -> Value {
        let matches = word_matches(&self.text)
            .into_iter()
            .map(|found| json!({"text": found.text, "end": found.end}))
            .collect();
        Value::Array(matches)
    }
}

#[derive(Deserialize)]
pub struct OpensWithGreetingRequest {
    pub text: String,
}

impl OpensWithGreetingRequest {
    #[must_use]
    pub fn answer(&self) -> Value {
        json!(opens_with_greeting(&self.text))
    }
}
