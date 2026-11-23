use super::error::RecordError;
use crate::listener::daemon::timing::Timing;

pub const TRANSCRIBER_RETRY_SECONDS: f64 = 1.0;
pub const TRANSCRIBER_ATTEMPTS: usize = 30;

#[derive(Debug, Clone, PartialEq)]
pub enum TranscribeFailure {
    Unreachable(String),
    Failed(String),
}

pub trait Transcriber {
    fn transcribe(
        &mut self,
        segment: &[u8],
        language: Option<&str>,
    ) -> Result<String, TranscribeFailure>;
}

fn settled(outcome: Result<String, TranscribeFailure>) -> Option<Result<String, RecordError>> {
    match outcome {
        Ok(text) => Some(Ok(text)),
        Err(TranscribeFailure::Failed(message)) => Some(Err(RecordError::Transcription(message))),
        Err(TranscribeFailure::Unreachable(_)) => None,
    }
}

fn after_failure(timing: &mut impl Timing) -> Option<Result<String, RecordError>> {
    timing.pause(TRANSCRIBER_RETRY_SECONDS);
    None
}

pub fn transcribe_when_ready(
    transcriber: &mut impl Transcriber,
    timing: &mut impl Timing,
    segment: &[u8],
    language: Option<&str>,
) -> Result<String, RecordError> {
    let attempt = |_| {
        let outcome = transcriber.transcribe(segment, language);
        settled(outcome).or_else(|| after_failure(timing))
    };
    (0..TRANSCRIBER_ATTEMPTS)
        .find_map(attempt)
        .unwrap_or(Err(RecordError::NoTranscriberAnswered))
}
