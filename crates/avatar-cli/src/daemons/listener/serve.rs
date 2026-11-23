use std::io;
use std::path::Path;
use std::sync::Arc;

use avatar_listening::listener::capture::StopFlag;
use avatar_listening::listener::daemon::{
    DaemonRequest, ListenerConfiguration, RecorderParts, SystemTiming, TranscriptRecorder,
    read_configuration, run_daemon, stop_on_sigterm,
};
use avatar_listening::listener::session::{
    CONFIGURATION_FILE, EMBEDDINGS_FILE, SPEECH_IN_PROGRESS_MARKER, TRANSCRIPT_FILE,
};
use avatar_listening::listener::speaker_identity::SpeakerIdentification;
use avatar_transcript::pending::PendingSpeech;
use avatar_transcript::pending::speech_marker_file::FileSpeechMarker;
use tokio::runtime::{Builder, Handle};

use super::identity::identifier_for;
use super::kit::ProductionKit;
use super::notes::StandardErrorNotes;
use super::transcriber::HexTranscriber;
use crate::support::runtime::session_directory;

type Pending = PendingSpeech<FileSpeechMarker>;
type Recorder = TranscriptRecorder<HexTranscriber, Box<dyn SpeakerIdentification>, SystemTiming>;
type Daemon = DaemonRequest<ProductionKit, Pending, StandardErrorNotes>;

fn recorder_of(
    directory: &Path,
    configuration: &ListenerConfiguration,
    runtime: &Handle,
) -> io::Result<Recorder> {
    Ok(TranscriptRecorder::new(RecorderParts {
        transcript_path: directory.join(TRANSCRIPT_FILE),
        embeddings_path: directory.join(EMBEDDINGS_FILE),
        language: configuration.language.clone(),
        transcriber: HexTranscriber::new(runtime.clone())?,
        identifier: identifier_for(configuration.identify_speakers),
        timing: SystemTiming,
    }))
}

fn daemon_of(
    directory: &Path,
    configuration: ListenerConfiguration,
    runtime: &Handle,
) -> io::Result<Daemon> {
    let stopping = StopFlag::new();
    stop_on_sigterm(stopping.clone())?;
    let marker = FileSpeechMarker::new(directory.join(SPEECH_IN_PROGRESS_MARKER));
    Ok(DaemonRequest {
        kit: ProductionKit::load(&configuration, runtime)?,
        pending: Arc::new(PendingSpeech::new(marker)?),
        stopping,
        notes: StandardErrorNotes,
        configuration,
    })
}

fn unreadable(failure: &io::Error) -> io::Error {
    io::Error::other(format!("{CONFIGURATION_FILE}: {failure}"))
}

pub(crate) fn run() -> io::Result<()> {
    let directory = session_directory();
    let configuration = read_configuration(&directory).map_err(|failure| unreadable(&failure))?;
    let runtime = Builder::new_multi_thread().enable_all().build()?;
    let mut recorder = recorder_of(&directory, &configuration, runtime.handle())?;
    let mut daemon = daemon_of(&directory, configuration, runtime.handle())?;
    run_daemon(&mut daemon, &mut recorder);
    Ok(())
}
