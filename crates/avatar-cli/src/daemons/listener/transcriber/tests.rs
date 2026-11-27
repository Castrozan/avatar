use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsString;
use std::fs;

use avatar_listening::listener::daemon::{TranscribeFailure, Transcriber};
use avatar_transcription::HexClient;
use tempfile::TempDir;
use tokio::runtime::{Builder, Runtime};

use super::CloudFirstTranscriber;
use super::scripted_sender::{HEX_TEXT, SCRIBE_TEXT, ScriptedSender};
use super::settings::scribe_client;

type TestResult = Result<(), Box<dyn Error>>;

const SEGMENT: [u8; 4] = [0, 1, 2, 3];
const SCRIBE_URL: &str = "https://api.elevenlabs.io/v1/speech-to-text";
const HEX_MODELS_URL: &str = "http://127.0.0.1:7/models";
const HEX_TRANSCRIPTION_URL: &str =
    "http://127.0.0.1:7/transcriptions?model=parakeet_v3&language=en";
const HEX_DISCOVERY: &str = r#"{"port":7,"token":"token"}"#;

struct Fixture {
    runtime: Runtime,
    directory: TempDir,
}

impl Fixture {
    fn new() -> Result<Self, Box<dyn Error>> {
        let directory = tempfile::tempdir()?;
        fs::write(directory.path().join("roster.txt"), "Ozymandias | a king\n")?;
        let runtime = Builder::new_multi_thread().enable_all().build()?;
        Ok(Self { runtime, directory })
    }

    fn with_hex_running(self) -> Result<Self, Box<dyn Error>> {
        fs::write(self.directory.path().join("local-api.json"), HEX_DISCOVERY)?;
        Ok(self)
    }

    fn environment(&self) -> HashMap<String, OsString> {
        let roster = self.directory.path().join("roster.txt");
        HashMap::from([
            ("ELEVENLABS_API_KEY".to_owned(), "test-key".into()),
            ("AVATAR_SERVANT_ROSTER".to_owned(), roster.into_os_string()),
        ])
    }

    fn hear(&self, sender: &ScriptedSender) -> Result<String, TranscribeFailure> {
        let cloud = scribe_client(sender.clone(), &self.environment());
        let discovery = self.directory.path().join("local-api.json");
        let local = HexClient::new(sender.clone(), discovery);
        let mut transcriber =
            CloudFirstTranscriber::new(self.runtime.handle().clone(), cloud, local);
        transcriber.transcribe(&SEGMENT, Some("en"))
    }
}

fn urls(sender: &ScriptedSender) -> Vec<String> {
    let requests = sender.requests();
    requests.into_iter().map(|request| request.url).collect()
}

#[test]
fn a_segment_goes_to_scribe_and_hex_is_not_asked_while_scribe_answers() -> TestResult {
    let sender = ScriptedSender::answering_scribe_with(200);
    let heard = Fixture::new()?.with_hex_running()?.hear(&sender);
    assert_eq!(heard, Ok(SCRIBE_TEXT.to_owned()));
    assert_eq!(urls(&sender), [SCRIBE_URL]);
    Ok(())
}

#[test]
fn hex_answers_only_after_scribe_fails() -> TestResult {
    let sender = ScriptedSender::answering_scribe_with(503);
    let heard = Fixture::new()?.with_hex_running()?.hear(&sender);
    assert_eq!(heard, Ok(HEX_TEXT.to_owned()));
    let expected = [SCRIBE_URL, HEX_MODELS_URL, HEX_TRANSCRIPTION_URL];
    assert_eq!(urls(&sender), expected);
    Ok(())
}

#[test]
fn a_failed_scribe_and_an_unreachable_hex_leave_the_segment_to_retry() -> TestResult {
    let sender = ScriptedSender::answering_scribe_with(503);
    let heard = Fixture::new()?.hear(&sender);
    assert!(matches!(heard, Err(TranscribeFailure::Unreachable(_))));
    Ok(())
}

#[test]
fn scribe_receives_the_environment_key_and_the_roster_keyterms() -> TestResult {
    let sender = ScriptedSender::answering_scribe_with(200);
    let heard = Fixture::new()?.with_hex_running()?.hear(&sender);
    assert!(heard.is_ok());
    let request = sender.requests().remove(0);
    let key = ("xi-api-key".to_owned(), "test-key".to_owned());
    assert!(request.headers.contains(&key));
    let body = String::from_utf8_lossy(&request.body.unwrap_or_default()).into_owned();
    assert!(body.contains("name=\"keyterms\"\r\n\r\nOzzie\r\n"));
    Ok(())
}
