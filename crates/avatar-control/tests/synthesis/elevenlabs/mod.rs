mod timeout;
mod vowels;

use std::path::PathBuf;

const NATURAL: &str = "natural and conversational";

use avatar_control::speech::spoken_language::SpokenLanguage;
use avatar_control::speech::spoken_vowels::{SpokenVowel, VowelShape};
use avatar_control::synthesis::contract::{SpeechRequest, SynthesisResult};
use avatar_control::synthesis::media_speech::MediaSpeechClient;
use avatar_control::synthesis::media_speech::elevenlabs::ElevenLabsSpeechGenerator;

use crate::harness::{
    TestResult, UNAVAILABLE_ALIGNMENT, first_call, has_pair, media_speech_program, request,
    success_body,
};
use avatar_control::synthesis::contract::SpeechGenerator;

fn generator(program: PathBuf) -> ElevenLabsSpeechGenerator {
    ElevenLabsSpeechGenerator::new(
        MediaSpeechClient::new(program),
        "cloudDefault".to_owned(),
        SpokenLanguage::PortugueseBrazil,
    )
}

fn vowel(shape: VowelShape, start: f64, end: f64) -> SpokenVowel {
    SpokenVowel { shape, start, end }
}

async fn argv_for(speech: SpeechRequest) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let program = media_speech_program(directory.path(), &success_body(UNAVAILABLE_ALIGNMENT))?;
    generator(program).synthesize(speech).await?;
    Ok(first_call(directory.path()))
}

async fn with_emotion(emotion: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut speech = request("oi");
    speech.emotion = Some(emotion.to_owned());
    argv_for(speech).await
}

async fn with_alignment(text: &str, alignment: &str) -> SynthesisResult {
    let directory = tempfile::tempdir()?;
    let program = media_speech_program(directory.path(), &success_body(alignment))?;
    generator(program).synthesize(request(text)).await
}

#[tokio::test]
async fn default_model_sends_the_emotion_directions() -> TestResult {
    let argv = with_emotion("happy").await?.join(" ");
    assert_eq!(
        argv,
        "generate --provider elevenlabs --model eleven_v4 --directions warm and cheerful, smiling --language pt-br --voice cloudDefault --text oi"
    );
    Ok(())
}

#[tokio::test]
async fn unknown_emotion_uses_natural_delivery() -> TestResult {
    assert!(has_pair(
        &with_emotion("bewildered").await?,
        "--directions",
        NATURAL
    ));
    Ok(())
}

#[tokio::test]
async fn missing_emotion_uses_natural_delivery() -> TestResult {
    assert!(has_pair(
        &argv_for(request("oi")).await?,
        "--directions",
        NATURAL
    ));
    Ok(())
}

#[tokio::test]
async fn other_models_omit_the_directions_argument() -> TestResult {
    let mut speech = request("oi");
    speech.emotion = Some("sad".to_owned());
    speech.model = Some("eleven_turbo_v2_5".to_owned());
    let argv = argv_for(speech).await?;
    assert!(!argv.contains(&"--directions".to_owned()));
    assert!(has_pair(&argv, "--model", "eleven_turbo_v2_5"));
    Ok(())
}

#[tokio::test]
async fn requested_language_and_voice_reach_the_program() -> TestResult {
    let mut speech = request("hello");
    speech.voice = Some("cloudOther".to_owned());
    speech.language = Some(SpokenLanguage::EnglishUs);
    let argv = argv_for(speech).await?;
    assert!(has_pair(&argv, "--language", "en-us"));
    assert!(has_pair(&argv, "--voice", "cloudOther"));
    Ok(())
}

#[tokio::test]
async fn elevenlabs_propagates_a_media_speech_failure() -> TestResult {
    let directory = tempfile::tempdir()?;
    let program = media_speech_program(directory.path(), "exit 1")?;
    assert!(generator(program).synthesize(request("oi")).await.is_err());
    Ok(())
}
