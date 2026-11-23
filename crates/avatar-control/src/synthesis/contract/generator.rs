use std::future::Future;

use super::{SpeechRequest, SynthesisResult};

pub trait SpeechGenerator: Send + Sync {
    fn synthesize(&self, request: SpeechRequest) -> impl Future<Output = SynthesisResult> + Send;
}
