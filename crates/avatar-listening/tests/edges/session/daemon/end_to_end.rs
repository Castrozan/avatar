use std::path::Path;
use std::sync::{Arc, Mutex};

use super::capture_kit::Kit;
use avatar_listening::listener::capture::StopFlag;
use avatar_listening::listener::daemon::{
    DaemonRequest, ListenerConfiguration, RecorderParts, SystemTiming, TranscribeFailure,
    Transcriber, TranscriptRecorder, run_daemon,
};
use avatar_listening::listener::diagnostics::Diagnostics;
use avatar_listening::listener::speaker_identity::DisabledIdentifier;
use avatar_transcript::pending::PendingSpeech;
use avatar_transcript::pending::speech_marker_file::FileSpeechMarker;

type Failure = Box<dyn std::error::Error>;
type Outcome = Result<(), Failure>;

const ENTRY_FIELDS: &str =
    r#""source": "human", "text": "hello there", "speaker": null, "turn_finished": true"#;

struct StopsAfterAnswering(StopFlag);

impl Transcriber for StopsAfterAnswering {
    fn transcribe(
        &mut self,
        _segment: &[u8],
        _language: Option<&str>,
    ) -> Result<String, TranscribeFailure> {
        self.0.set();
        Ok("hello there".to_owned())
    }
}

#[derive(Clone, Default)]
struct SharedNotes(Arc<Mutex<Vec<String>>>);

impl Diagnostics for SharedNotes {
    fn note(&mut self, line: &str) {
        self.0
            .lock()
            .map(|mut lines| lines.push(line.to_owned()))
            .ok();
    }
}

fn speech_then_silence() -> Vec<Vec<u8>> {
    let loud: Vec<u8> = std::iter::repeat_n(6000_i16.to_le_bytes(), 512)
        .flatten()
        .collect();
    let mut frames = vec![loud; 14];
    frames.extend(vec![vec![0; 1024]; 30]);
    frames
}

fn configuration() -> ListenerConfiguration {
    ListenerConfiguration {
        sources: [("human".to_owned(), "mic".to_owned())].into(),
        barge_in: false,
        language: None,
        identify_speakers: false,
    }
}

type Request = DaemonRequest<Kit, PendingSpeech<FileSpeechMarker>, SharedNotes>;

fn request_in(
    directory: &Path,
    stopping: &StopFlag,
    notes: SharedNotes,
) -> Result<Request, Failure> {
    let marker = FileSpeechMarker::new(directory.join("speech-in-progress"));
    Ok(DaemonRequest {
        kit: Kit(speech_then_silence()),
        configuration: configuration(),
        pending: Arc::new(PendingSpeech::new(marker)?),
        stopping: stopping.clone(),
        notes,
    })
}

type Recorder = TranscriptRecorder<StopsAfterAnswering, DisabledIdentifier, SystemTiming>;

fn recorder_in(directory: &Path, stopping: &StopFlag) -> Recorder {
    TranscriptRecorder::new(RecorderParts {
        transcript_path: directory.join("transcript.jsonl"),
        embeddings_path: directory.join("embeddings.jsonl"),
        language: None,
        transcriber: StopsAfterAnswering(stopping.clone()),
        identifier: DisabledIdentifier,
        timing: SystemTiming,
    })
}

#[test]
fn spoken_audio_becomes_a_transcript_entry_and_a_stop_ends_the_daemon() -> Outcome {
    let directory = tempfile::tempdir()?;
    let (stopping, notes) = (StopFlag::new(), SharedNotes::default());
    let mut request = request_in(directory.path(), &stopping, notes.clone())?;
    run_daemon(&mut request, &mut recorder_in(directory.path(), &stopping));
    let written = std::fs::read_to_string(directory.path().join("transcript.jsonl"))?;
    assert_eq!(written.lines().count(), 1, "{written}");
    assert!(written.contains(ENTRY_FIELDS), "{written}");
    assert!(!directory.path().join("speech-in-progress").exists());
    assert!(notes.0.lock().map_err(|_| "notes")?.is_empty());
    Ok(())
}
