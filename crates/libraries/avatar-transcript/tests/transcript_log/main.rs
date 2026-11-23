mod details;
mod properties;

use std::error::Error;

use avatar_transcript::transcript::store::FileTranscriptStore;
use avatar_transcript::transcript::{Transcript, Utterance};
use details::details_of;
use serde::Deserialize;
use serde_json::Value;

const FIXTURE: &str = include_str!("fixtures/transcript_log_cases.json");
const SEEDED_TRANSCRIPT: &str = concat!(
    r#"{"sequence": 1, "time": 1.0, "source": "a", "text": "x", "speaker": null, "turn_finished": true}"#,
    "\n",
    r#"{"sequence": 2, "time": 2.0, "source": "b", "text": "y", "speaker": null, "turn_finished": true}"#,
    "\n",
    r#"{"sequence": 3, "time": 3.0, "source": "a", "text": "z", "speaker": null, "turn_finished": true}"#,
);

#[derive(Deserialize)]
struct TranscriptCases {
    append: Vec<AppendCase>,
    read: ReadCases,
}

#[derive(Deserialize)]
struct AppendCase {
    time: f64,
    source: String,
    text: String,
    speaker: Option<String>,
    details: Vec<(String, Value)>,
    line: String,
}

#[derive(Deserialize)]
struct ReadCases {
    since_0: Vec<i64>,
    since_1: Vec<i64>,
    since_3: Vec<i64>,
    last: i64,
    missing_last: i64,
    missing_since: Vec<i64>,
}

#[test]
fn appended_lines_match_python_byte_for_byte() -> Result<(), Box<dyn Error>> {
    let cases: TranscriptCases = serde_json::from_str(FIXTURE)?;
    for case in cases.append {
        replay_append(&case)?;
    }
    Ok(())
}

fn replay_append(case: &AppendCase) -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("transcript.jsonl");
    let transcript = Transcript::new(FileTranscriptStore::new(&path));
    transcript.append(utterance_of(case), case.time, &details_of(&case.details))?;
    assert_eq!(std::fs::read_to_string(&path)?, case.line);
    Ok(())
}

fn utterance_of(case: &AppendCase) -> Utterance {
    Utterance {
        source: case.source.clone(),
        text: case.text.clone(),
        speaker: case.speaker.clone(),
    }
}

#[test]
fn reads_and_sequences_match_python() -> Result<(), Box<dyn Error>> {
    let cases: TranscriptCases = serde_json::from_str(FIXTURE)?;
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("transcript.jsonl");
    std::fs::write(&path, SEEDED_TRANSCRIPT)?;
    let transcript = Transcript::new(FileTranscriptStore::new(&path));
    assert_reads(&transcript, &cases.read)
}

fn assert_reads(
    transcript: &Transcript<FileTranscriptStore>,
    reads: &ReadCases,
) -> Result<(), Box<dyn Error>> {
    assert_eq!(sequences(&transcript.read_since(0)?), reads.since_0);
    assert_eq!(sequences(&transcript.read_since(1)?), reads.since_1);
    assert_eq!(sequences(&transcript.read_since(3)?), reads.since_3);
    assert_eq!(transcript.last_sequence()?, reads.last);
    Ok(())
}

#[test]
fn a_missing_file_reads_as_an_empty_transcript() -> Result<(), Box<dyn Error>> {
    let cases: TranscriptCases = serde_json::from_str(FIXTURE)?;
    let directory = tempfile::tempdir()?;
    let absent = directory.path().join("absent.jsonl");
    let transcript = Transcript::new(FileTranscriptStore::new(absent));
    assert_eq!(transcript.last_sequence()?, cases.read.missing_last);
    assert_eq!(
        sequences(&transcript.read_since(0)?),
        cases.read.missing_since
    );
    Ok(())
}

fn sequences(entries: &[Value]) -> Vec<i64> {
    entries
        .iter()
        .filter_map(|entry| entry.get("sequence").and_then(Value::as_i64))
        .collect()
}
