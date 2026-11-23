use std::path::PathBuf;

use avatar_control::speech::spoken_language::SpokenLanguage;
use avatar_control::synthesis::contract::{SpeechGenerator, SpeechRequest, SpeechSource};
use avatar_control::synthesis::media_speech::MediaSpeechClient;
use avatar_control::synthesis::media_speech::kokoro::KokoroSpeechGenerator;

use crate::harness::{
    TestResult, UNAVAILABLE_ALIGNMENT, first_call, has_pair, media_speech_program, request,
    success_body,
};

fn generator(program: PathBuf, voice: &str) -> KokoroSpeechGenerator {
    KokoroSpeechGenerator::new(
        MediaSpeechClient::new(program),
        voice.to_owned(),
        SpokenLanguage::PortugueseBrazil,
    )
}

async fn argv_for(speech: SpeechRequest) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let program = media_speech_program(directory.path(), &success_body(UNAVAILABLE_ALIGNMENT))?;
    generator(program, "pf_dora").synthesize(speech).await?;
    Ok(first_call(directory.path()))
}

#[tokio::test]
async fn kokoro_argument_vector_matches_the_javascript_builder() -> TestResult {
    let argv = argv_for(request("oi")).await?;
    assert_eq!(
        argv.join(" "),
        "generate --provider kokoro --language pt-br --voice pf_dora --text oi"
    );
    Ok(())
}

#[tokio::test]
async fn kokoro_uses_the_requested_voice_and_language() -> TestResult {
    let mut speech = request("hello");
    speech.voice = Some("am_x".to_owned());
    speech.language = Some(SpokenLanguage::EnglishUs);
    let argv = argv_for(speech).await?;
    assert!(has_pair(&argv, "--voice", "am_x"));
    assert!(has_pair(&argv, "--language", "en-us"));
    Ok(())
}

#[tokio::test]
async fn kokoro_falls_back_to_the_default_voice_when_the_voice_is_empty() -> TestResult {
    let mut speech = request("oi");
    speech.voice = Some(String::new());
    assert!(has_pair(&argv_for(speech).await?, "--voice", "pf_dora"));
    Ok(())
}

#[tokio::test]
async fn kokoro_result_has_kokoro_source_and_no_vowels() -> TestResult {
    let directory = tempfile::tempdir()?;
    let program = media_speech_program(directory.path(), &success_body(UNAVAILABLE_ALIGNMENT))?;
    let speech = generator(program, "pf_dora")
        .synthesize(request("oi"))
        .await?;
    assert_eq!(speech.source, SpeechSource::Kokoro);
    assert!(speech.vowels.is_empty());
    assert!((speech.duration_seconds - 1.5).abs() < f64::EPSILON);
    Ok(())
}

#[tokio::test]
async fn kokoro_propagates_a_media_speech_failure() -> TestResult {
    let directory = tempfile::tempdir()?;
    let program = media_speech_program(directory.path(), "exit 1")?;
    assert!(
        generator(program, "pf_dora")
            .synthesize(request("oi"))
            .await
            .is_err()
    );
    Ok(())
}
