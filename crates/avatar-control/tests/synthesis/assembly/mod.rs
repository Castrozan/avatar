mod fixture;

use avatar_control::speech::spoken_language::SpokenLanguage;
use avatar_control::synthesis::assembly::language_from_tag;
use avatar_control::synthesis::character::CharacterVoice;
use avatar_control::synthesis::contract::{SpeechGenerator, SpeechSource};

use crate::harness::{
    TestResult, UNAVAILABLE_ALIGNMENT, calls, media_speech_program, request, success_body,
};
use fixture::{failing_providers, stack_for};

#[tokio::test]
async fn assembled_stack_sends_the_cloud_request_first() -> TestResult {
    let directory = tempfile::tempdir()?;
    let media_speech =
        media_speech_program(directory.path(), &success_body(UNAVAILABLE_ALIGNMENT))?;
    let stack = stack_for(directory.path(), media_speech)?;
    assert_eq!(
        stack.synthesize(request("oi")).await?.source,
        SpeechSource::ElevenLabs
    );
    assert!(calls(directory.path())[0].contains(&"cloudFirst".to_owned()));
    Ok(())
}

#[tokio::test]
async fn cloud_and_kokoro_failures_fall_through_to_piper() -> TestResult {
    let directory = tempfile::tempdir()?;
    let media_speech = failing_providers(directory.path(), "elevenlabs|kokoro")?;
    let stack = stack_for(directory.path(), media_speech)?;
    assert_eq!(
        stack.synthesize(request("olá")).await?.source,
        SpeechSource::Piper
    );
    assert_eq!(calls(directory.path()).len(), 2);
    Ok(())
}

#[test]
fn language_tags_map_to_the_two_supported_languages() {
    assert_eq!(
        language_from_tag("pt-br").ok(),
        Some(SpokenLanguage::PortugueseBrazil)
    );
    assert_eq!(
        language_from_tag("en-us").ok(),
        Some(SpokenLanguage::EnglishUs)
    );
}

#[test]
fn unsupported_language_tags_are_rejected() {
    assert!(language_from_tag("fr-fr").is_err());
}

#[test]
fn character_voice_file_uses_camel_case_keys() {
    let text = r#"[{"name":"first","cloudVoice":"voiceA","localVoice":"pf_first"}]"#;
    let characters = serde_json::from_str::<Vec<CharacterVoice>>(text).unwrap_or_default();
    let expected = vec![CharacterVoice {
        name: "first".to_owned(),
        cloud_voice: Some("voiceA".to_owned()),
        local_voice: Some("pf_first".to_owned()),
    }];
    assert_eq!(characters, expected);
}
