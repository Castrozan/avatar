mod fixture;
mod local;

use crate::harness::{TestResult, request};
use fixture::run;

#[tokio::test]
async fn kokoro_voice_is_routed_to_kokoro_only() -> TestResult {
    let mut speech = request("hello");
    speech.voice = Some("pf_dora".to_owned());
    let (audio, calls) = run(speech, false, false).await;
    assert_eq!(audio.as_deref(), Some("kokoro"));
    assert_eq!(calls.len(), 1);
    assert!(calls[0].starts_with("kokoro "));
    Ok(())
}

#[tokio::test]
async fn piper_voice_is_routed_to_piper_only_with_its_output_id() -> TestResult {
    let mut speech = request("olá");
    speech.voice = Some("pt_BR-faber-medium".to_owned());
    let (audio, calls) = run(speech, false, false).await;
    assert_eq!(audio.as_deref(), Some("piper"));
    assert_eq!(calls.len(), 1);
    assert!(calls[0].contains("output_id: \"turn\""));
    Ok(())
}

#[tokio::test]
async fn cloud_failure_falls_back_to_the_home_kokoro_voice() -> TestResult {
    let (audio, calls) = run(request("olá"), true, false).await;
    assert_eq!(audio.as_deref(), Some("kokoro"));
    assert_eq!(calls.len(), 2);
    assert!(calls[1].contains("voice: Some(\"pf_dora\")"));
    assert!(calls[1].contains("language: Some(PortugueseBrazil)"));
    Ok(())
}

#[tokio::test]
async fn kokoro_failure_falls_back_to_the_piper_voice() -> TestResult {
    let (audio, calls) = run(request("olá"), true, true).await;
    assert_eq!(audio.as_deref(), Some("piper"));
    assert_eq!(calls.len(), 3);
    assert!(calls[2].contains("voice: Some(\"faber\")"));
    Ok(())
}

#[tokio::test]
async fn text_in_no_speakable_script_gets_no_fallback() -> TestResult {
    let (audio, calls) = run(request("こんにちは"), true, false).await;
    assert_eq!(audio, None);
    assert_eq!(calls.len(), 1);
    assert!(calls[0].contains("language: Some(EnglishUs)"));
    Ok(())
}

#[tokio::test]
async fn cloud_receives_voice_emotion_model_and_language() -> TestResult {
    let mut speech = request("hello");
    speech.voice = Some("cloudVoice".to_owned());
    speech.emotion = Some("happy".to_owned());
    speech.model = Some("eleven_flash_v2_5".to_owned());
    let (_, calls) = run(speech, false, false).await;
    assert!(calls[0].contains("voice: Some(\"cloudVoice\")"));
    assert!(calls[0].contains("emotion: Some(\"happy\")"));
    assert!(calls[0].contains("model: Some(\"eleven_flash_v2_5\")"));
    assert!(calls[0].contains("language: Some(EnglishUs)"));
    Ok(())
}
