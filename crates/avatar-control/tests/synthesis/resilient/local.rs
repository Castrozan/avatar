use crate::harness::{TestResult, request};

use super::fixture::run;

#[tokio::test]
async fn character_local_voice_overrides_the_home_kokoro_voice() -> TestResult {
    let mut speech = request("olá");
    speech.local_voice = Some("pm_alex".to_owned());
    let (_, calls) = run(speech, true, false).await;
    assert!(calls[1].contains("voice: Some(\"pm_alex\")"));
    Ok(())
}

#[tokio::test]
async fn english_fallback_uses_english_voices_and_ignores_the_character_voice() -> TestResult {
    let mut speech = request("hello there, how are you");
    speech.local_voice = Some("pm_alex".to_owned());
    let (_, calls) = run(speech, true, false).await;
    assert!(calls[1].contains("voice: Some(\"af_heart\")"));
    assert!(calls[1].contains("language: Some(EnglishUs)"));
    Ok(())
}
