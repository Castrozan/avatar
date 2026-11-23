mod fixture;

use avatar_control::synthesis::contract::{SpeechSource, SynthesisError};
use avatar_control::synthesis::piper::prepare_output_root_directory;

use crate::harness::{TestResult, request};
use avatar_control::synthesis::contract::{SpeechGenerator, SpeechRequest};
use fixture::{copying_fixture, expected_arguments, generator, piper_program};

#[tokio::test]
async fn piper_receives_the_model_output_file_and_text() -> TestResult {
    let directory = tempfile::tempdir()?;
    let program = copying_fixture(directory.path())?;
    generator(directory.path(), program)
        .synthesize(request("olá"))
        .await?;
    let arguments = std::fs::read_to_string(directory.path().join("piper-args.txt"))?;
    assert_eq!(arguments, expected_arguments(directory.path()));
    Ok(())
}

#[tokio::test]
async fn piper_writes_the_text_to_standard_input() -> TestResult {
    let directory = tempfile::tempdir()?;
    generator(directory.path(), copying_fixture(directory.path())?)
        .synthesize(request("olá"))
        .await?;
    assert_eq!(
        std::fs::read_to_string(directory.path().join("piper-stdin.txt"))?,
        "olá"
    );
    Ok(())
}

#[tokio::test]
async fn piper_reports_the_duration_from_the_wav_header() -> TestResult {
    let directory = tempfile::tempdir()?;
    let speech = generator(directory.path(), copying_fixture(directory.path())?)
        .synthesize(request("olá"))
        .await?;
    assert!((speech.duration_seconds - 1.0).abs() < f64::EPSILON);
    assert_eq!(speech.source, SpeechSource::Piper);
    assert!(speech.vowels.is_empty());
    Ok(())
}

#[tokio::test]
async fn piper_rejects_a_voice_name_with_path_characters_before_running() -> TestResult {
    let directory = tempfile::tempdir()?;
    let mut speech: SpeechRequest = request("olá");
    speech.voice = Some("../escape".to_owned());
    let result = generator(directory.path(), copying_fixture(directory.path())?)
        .synthesize(speech)
        .await;
    assert!(matches!(result, Err(SynthesisError::UnknownVoice(ref voice)) if voice == "../escape"));
    assert!(!directory.path().join("piper-args.txt").exists());
    Ok(())
}

#[tokio::test]
async fn piper_nonzero_exit_is_a_failure() -> TestResult {
    let directory = tempfile::tempdir()?;
    let program = piper_program(directory.path(), "exit 2")?;
    assert!(
        generator(directory.path(), program)
            .synthesize(request("olá"))
            .await
            .is_err()
    );
    Ok(())
}

#[tokio::test]
async fn truncated_audio_is_a_failure() -> TestResult {
    let directory = tempfile::tempdir()?;
    let program = piper_program(directory.path(), "printf 'short' > \"$out\"")?;
    assert!(
        generator(directory.path(), program)
            .synthesize(request("olá"))
            .await
            .is_err()
    );
    Ok(())
}

#[tokio::test]
async fn output_root_directory_is_created_private() -> TestResult {
    use std::os::unix::fs::PermissionsExt;
    let directory = tempfile::tempdir()?;
    let root = directory.path().join("tts");
    prepare_output_root_directory(&root).await?;
    assert_eq!(
        std::fs::metadata(&root)?.permissions().mode() & 0o777,
        0o700
    );
    Ok(())
}
