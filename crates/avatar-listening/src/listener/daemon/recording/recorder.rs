use std::path::PathBuf;

use avatar_transcript::echo::EchoFilter;
use avatar_transcript::transcript::Transcript;
use avatar_transcript::transcript::store::FileTranscriptStore;

use super::entry_details::{details_of, non_empty, sequence_of, utterance_of, without_echo};
use super::error::RecordError;
use super::transcriber::{Transcriber, transcribe_when_ready};
use crate::listener::daemon::segments::QueuedSegment;
use crate::listener::daemon::timing::Timing;
use crate::listener::diagnostics::Diagnostics;
use crate::listener::speaker_identity::{SpeakerIdentification, append_embedding};

pub trait SegmentRecorder {
    fn record(
        &mut self,
        segment: &QueuedSegment,
        notes: &mut dyn Diagnostics,
    ) -> Result<(), RecordError>;
}

pub struct TranscriptRecorder<Engine, Identity, Clock> {
    transcript: Transcript<FileTranscriptStore>,
    embeddings_path: PathBuf,
    language: Option<String>,
    echoes: EchoFilter,
    transcriber: Engine,
    identifier: Identity,
    timing: Clock,
}

pub struct RecorderParts<Engine, Identity, Clock> {
    pub transcript_path: PathBuf,
    pub embeddings_path: PathBuf,
    pub language: Option<String>,
    pub transcriber: Engine,
    pub identifier: Identity,
    pub timing: Clock,
}

impl<Engine, Identity, Clock> TranscriptRecorder<Engine, Identity, Clock> {
    pub fn new(parts: RecorderParts<Engine, Identity, Clock>) -> Self {
        Self {
            transcript: Transcript::new(FileTranscriptStore::new(parts.transcript_path)),
            embeddings_path: parts.embeddings_path,
            language: parts.language,
            echoes: EchoFilter::default(),
            transcriber: parts.transcriber,
            identifier: parts.identifier,
            timing: parts.timing,
        }
    }
}

impl<Engine: Transcriber, Identity: SpeakerIdentification, Clock: Timing>
    TranscriptRecorder<Engine, Identity, Clock>
{
    fn fresh_text(&mut self, segment: &QueuedSegment) -> Result<Option<String>, RecordError> {
        let language = self.language.as_deref();
        let audio = &segment.audio;
        let engine = &mut self.transcriber;
        let text = transcribe_when_ready(engine, &mut self.timing, audio, language)?;
        let (echoes, timing, source) = (&mut self.echoes, &mut self.timing, &segment.source);
        Ok(non_empty(text).and_then(|text| without_echo(echoes, timing, source, text)))
    }

    fn store(
        &mut self,
        segment: &QueuedSegment,
        text: String,
        started: f64,
        notes: &mut dyn Diagnostics,
    ) -> Result<(), RecordError> {
        let heard = self.identifier.identify(&segment.audio, notes);
        let utterance = utterance_of(segment, text, heard.speaker);
        let now = self.timing.wall_seconds();
        let details = details_of(segment, started);
        let entry = self.transcript.append(utterance, now, &details)?;
        self.keep_embedding(sequence_of(&entry), &heard.embedding)
    }

    fn keep_embedding(&self, sequence: i64, embedding: &[f64]) -> Result<(), RecordError> {
        let keep = || append_embedding(&self.embeddings_path, sequence, embedding);
        (!embedding.is_empty())
            .then(keep)
            .transpose()
            .map_err(RecordError::Embeddings)?;
        Ok(())
    }
}

impl<Engine: Transcriber, Identity: SpeakerIdentification, Clock: Timing> SegmentRecorder
    for TranscriptRecorder<Engine, Identity, Clock>
{
    fn record(
        &mut self,
        segment: &QueuedSegment,
        notes: &mut dyn Diagnostics,
    ) -> Result<(), RecordError> {
        let started = self.timing.wall_seconds();
        let text = self.fresh_text(segment)?;
        text.map(|text| self.store(segment, text, started, notes))
            .transpose()?;
        Ok(())
    }
}
