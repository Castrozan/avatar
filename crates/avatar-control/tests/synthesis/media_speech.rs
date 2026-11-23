use std::time::{Duration, Instant};

use avatar_control::synthesis::contract::SynthesisError;
use avatar_control::synthesis::media_speech::MediaSpeechClient;

use crate::harness::{
    TestResult, UNAVAILABLE_ALIGNMENT, calls, media_speech_program, owned, success_body,
};

fn client(directory: &std::path::Path, body: &str) -> std::io::Result<MediaSpeechClient> {
    Ok(MediaSpeechClient::new(media_speech_program(
        directory, body,
    )?))
}

#[tokio::test]
async fn client_sends_the_argument_vector_unchanged() -> TestResult {
    let directory = tempfile::tempdir()?;
    let client = client(directory.path(), &success_body(UNAVAILABLE_ALIGNMENT))?;
    client
        .run(&["generate", "--text", "olá mundo"], Duration::from_secs(5))
        .await?;
    assert_eq!(
        calls(directory.path()),
        vec![owned(&["generate", "--text", "olá mundo"])]
    );
    Ok(())
}

#[tokio::test]
async fn client_parses_the_audio_path_and_duration() -> TestResult {
    let directory = tempfile::tempdir()?;
    let client = client(directory.path(), &success_body(UNAVAILABLE_ALIGNMENT))?;
    let output = client.run(&["generate"], Duration::from_secs(5)).await?;
    assert_eq!(output.audio.path.to_string_lossy(), "/speech.wav");
    assert!((output.audio.duration_seconds - 1.5).abs() < f64::EPSILON);
    Ok(())
}

#[tokio::test]
async fn client_reports_standard_error_when_the_program_exits_nonzero() -> TestResult {
    let directory = tempfile::tempdir()?;
    let client = client(directory.path(), "echo ' quota exceeded ' >&2\nexit 1")?;
    let result = client.run(&["generate"], Duration::from_secs(5)).await;
    assert!(
        matches!(result, Err(SynthesisError::Failed(ref detail)) if detail == "media-speech failed: quota exceeded")
    );
    Ok(())
}

#[tokio::test]
async fn client_rejects_malformed_standard_output() -> TestResult {
    let directory = tempfile::tempdir()?;
    let client = client(directory.path(), "printf 'not json'")?;
    let result = client.run(&["generate"], Duration::from_secs(5)).await;
    assert!(matches!(result, Err(SynthesisError::MalformedOutput(_))));
    Ok(())
}

#[tokio::test]
async fn client_reports_a_missing_program() -> TestResult {
    let directory = tempfile::tempdir()?;
    let client = MediaSpeechClient::new(directory.path().join("absent-media-speech"));
    let result = client.run(&["generate"], Duration::from_secs(5)).await;
    assert!(
        matches!(result, Err(SynthesisError::Failed(ref detail)) if detail.contains("failed to start"))
    );
    Ok(())
}

#[tokio::test]
async fn client_stops_a_hung_program_within_its_timeout() -> TestResult {
    let directory = tempfile::tempdir()?;
    let client = client(directory.path(), "exec sleep 30")?;
    let started = Instant::now();
    let result = client.run(&["generate"], Duration::from_millis(200)).await;
    assert!(result.is_err());
    assert!(started.elapsed() < Duration::from_secs(10));
    Ok(())
}

fn stack_dump_script(signal_log: &std::path::Path) -> String {
    let trap = format!(
        "trap 'echo dumped >> {}; exit 3' USR1",
        signal_log.display()
    );
    format!("{trap}\nwhile true; do sleep 0.05; done")
}

#[tokio::test]
async fn client_sends_the_stack_dump_signal_before_the_run_is_abandoned() -> TestResult {
    let directory = tempfile::tempdir()?;
    let signal_log = directory.path().join("signals.txt");
    let client = client(directory.path(), &stack_dump_script(&signal_log))?;
    let result = client.run(&["generate"], Duration::from_millis(200)).await;
    assert!(result.is_err());
    assert_eq!(std::fs::read_to_string(signal_log)?, "dumped\n");
    Ok(())
}
