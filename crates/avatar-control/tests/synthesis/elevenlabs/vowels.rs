use avatar_control::speech::spoken_vowels::VowelShape;
use avatar_control::synthesis::contract::SpeechSource;

use crate::harness::{TestResult, UNAVAILABLE_ALIGNMENT};

use super::{vowel, with_alignment};

#[tokio::test]
async fn available_alignment_yields_spoken_vowels() -> TestResult {
    let alignment = r#"{"status":"available","characters":["o","i"],"starts_seconds":[0.0,0.5],"ends_seconds":[0.4,0.9]}"#;
    let speech = with_alignment("oi", alignment).await?;
    assert_eq!(
        speech.vowels,
        vec![
            vowel(VowelShape::Oh, 0.0, 0.4),
            vowel(VowelShape::Ih, 0.5, 0.9)
        ]
    );
    Ok(())
}

#[tokio::test]
async fn delivery_tags_are_skipped_when_reading_vowels() -> TestResult {
    let alignment = r#"{"status":"available","characters":["[","w","a","r","m","]"," ","S","ó"],"starts_seconds":[0,1,2,3,4,5,6,7,8],"ends_seconds":[1,2,3,4,5,6,7,8,9]}"#;
    let speech = with_alignment("[warm] Só", alignment).await?;
    assert_eq!(speech.vowels, vec![vowel(VowelShape::Oh, 8.0, 9.0)]);
    Ok(())
}

#[tokio::test]
async fn unavailable_alignment_yields_no_vowels() -> TestResult {
    let speech = with_alignment("oi", UNAVAILABLE_ALIGNMENT).await?;
    assert!(speech.vowels.is_empty());
    assert_eq!(speech.source, SpeechSource::ElevenLabs);
    Ok(())
}
