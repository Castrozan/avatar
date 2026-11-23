use std::error::Error;
use std::io;
use std::path::Path;

use avatar_transcript::pending::PendingSpeech;
use avatar_transcript::pending::speech_marker_file::FileSpeechMarker;
use serde::Deserialize;

const FIXTURE: &str = include_str!("fixtures/pending_cases.json");

#[derive(Deserialize)]
struct PendingCase {
    program: Vec<String>,
    states: Vec<State>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct State {
    count: i64,
    marker: bool,
}

type Speech = PendingSpeech<FileSpeechMarker>;

fn run_operation(speech: &Speech, operation: &str) -> io::Result<()> {
    match operation {
        "begin" => speech.begin(),
        "finish" => speech.finish(),
        "clear" => speech.clear(),
        _ => unreachable!("fixture names only known operations"),
    }
}

fn state_of(speech: &Speech, marker_path: &Path) -> State {
    State {
        count: speech.count(),
        marker: marker_path.exists(),
    }
}

fn replay(case: &PendingCase) -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let marker_path = directory.path().join("speaking");
    std::fs::write(&marker_path, "stale")?;
    let speech = Speech::new(FileSpeechMarker::new(&marker_path))?;
    assert_eq!(state_of(&speech, &marker_path), case.states[0]);
    run_program(&speech, &marker_path, case)
}

fn run_program(
    speech: &Speech,
    marker_path: &Path,
    case: &PendingCase,
) -> Result<(), Box<dyn Error>> {
    for (index, operation) in case.program.iter().enumerate() {
        run_operation(speech, operation)?;
        assert_eq!(state_of(speech, marker_path), case.states[index + 1]);
    }
    Ok(())
}

#[test]
fn pending_speech_states_match_python() -> Result<(), Box<dyn Error>> {
    let cases: Vec<PendingCase> = serde_json::from_str(FIXTURE)?;
    for case in &cases {
        replay(case)?;
    }
    Ok(())
}
