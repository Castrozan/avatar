use std::error::Error;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::Mutex;

use avatar_control::synthesis::contract::{
    GeneratedSpeech, SpeechGenerator, SpeechRequest, SpeechSource, SynthesisError, SynthesisResult,
};

pub(crate) type TestResult = Result<(), Box<dyn Error>>;

const CALL_SEPARATOR: &str = "---";
pub(crate) const UNAVAILABLE_ALIGNMENT: &str = r#"{"status":"unavailable"}"#;
const WAV_HEADER_BYTES: usize = 44;
pub(crate) const WAV_BYTE_RATE: u32 = 32_000;
const WAV_DATA_BYTES: usize = 32_000;

pub(crate) fn write_script(path: &Path, body: &str) -> std::io::Result<()> {
    fs::write(path, format!("#!/bin/sh\n{body}\n"))?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))
}

pub(crate) fn recording_script(directory: &Path, body: &str) -> String {
    let log = directory.join("calls.txt").display().to_string();
    format!("printf '%s\\n' \"$@\" >> '{log}'\necho '{CALL_SEPARATOR}' >> '{log}'\n{body}")
}

pub(crate) fn media_speech_program(directory: &Path, body: &str) -> std::io::Result<PathBuf> {
    let path = directory.join("media-speech");
    write_script(&path, &recording_script(directory, body))?;
    Ok(path)
}

pub(crate) fn success_body(alignment_json: &str) -> String {
    format!(
        "printf '%s' '{{\"audio\":{{\"path\":\"/speech.wav\",\"duration_seconds\":1.5}},\"alignment\":{alignment_json}}}'"
    )
}

pub(crate) fn calls(directory: &Path) -> Vec<Vec<String>> {
    let text = fs::read_to_string(directory.join("calls.txt")).unwrap_or_default();
    let separator = format!("{CALL_SEPARATOR}\n");
    text.split(separator.as_str())
        .filter(|chunk| !chunk.is_empty())
        .map(|chunk| chunk.lines().map(str::to_owned).collect())
        .collect()
}

pub(crate) fn first_call(directory: &Path) -> Vec<String> {
    calls(directory).into_iter().next().unwrap_or_default()
}

pub(crate) fn has_pair(argv: &[String], flag: &str, value: &str) -> bool {
    argv.windows(2)
        .any(|pair| pair[0] == flag && pair[1] == value)
}

pub(crate) fn wav_fixture() -> Vec<u8> {
    let mut audio = vec![0_u8; WAV_HEADER_BYTES + WAV_DATA_BYTES];
    audio[28..32].copy_from_slice(&WAV_BYTE_RATE.to_le_bytes());
    audio
}

pub(crate) fn owned(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

pub(crate) fn request(text: &str) -> SpeechRequest {
    SpeechRequest {
        text: text.to_owned(),
        output_id: "turn".to_owned(),
        ..SpeechRequest::default()
    }
}

pub(crate) type Log = Arc<Mutex<Vec<String>>>;

pub(crate) fn new_log() -> Log {
    Arc::new(Mutex::new(Vec::new()))
}

pub(crate) struct Recorder {
    pub(crate) label: &'static str,
    pub(crate) log: Log,
    pub(crate) fails: bool,
}

impl SpeechGenerator for Recorder {
    async fn synthesize(&self, request: SpeechRequest) -> SynthesisResult {
        self.log
            .lock()
            .await
            .push(format!("{} {:?}", self.label, request));
        recorded_outcome(self.label, self.fails)
    }
}

fn recorded_outcome(label: &'static str, fails: bool) -> SynthesisResult {
    if fails {
        return Err(SynthesisError::Failed(format!("{label} is down")));
    }
    Ok(GeneratedSpeech {
        audio_path: PathBuf::from(label),
        duration_seconds: 1.0,
        vowels: Vec::new(),
        source: SpeechSource::Kokoro,
    })
}
