use std::collections::HashMap;

use crate::session::ClientId;
use crate::speaking::runtime::channel::ChannelRelease;
use crate::speaking::runtime::players::{PlaybackTarget, RunningPlayback, StopAction};
use crate::speaking::{SpeakRequest, SpeechTiming};
use crate::speech::chunk_sequence::SpeechChunkSequence;
use crate::synthesis::contract::{GeneratedSpeech, SpeechRequest, SpeechSource};

#[derive(Default)]
pub(super) struct Progress {
    pub(super) timing: SpeechTiming,
    pub(super) sources: Vec<SpeechSource>,
    pub(super) failure: Option<String>,
    pub(super) chunk_error: Option<String>,
    pub(super) sequence: Option<SpeechChunkSequence>,
    pub(super) ready: HashMap<usize, GeneratedSpeech>,
    pub(super) stops: Vec<StopAction>,
    pub(super) release: Option<ChannelRelease>,
}

impl Progress {
    fn received_at(received_ms: u64) -> Self {
        Self {
            timing: SpeechTiming::received_at(received_ms),
            ..Self::default()
        }
    }
}

pub(super) struct Turn {
    pub(super) client: ClientId,
    pub(super) request: SpeakRequest,
    pub(super) chunks: Vec<String>,
    pub(super) targets: Vec<PlaybackTarget>,
    pub(super) progress: Progress,
}

impl Turn {
    pub(super) fn new(
        client: ClientId,
        request: SpeakRequest,
        chunks: Vec<String>,
        targets: Vec<PlaybackTarget>,
        received_ms: u64,
    ) -> Self {
        Self {
            client,
            request,
            chunks,
            targets,
            progress: Progress::received_at(received_ms),
        }
    }

    pub(super) fn begin_synthesis(&mut self, index: usize, now_ms: u64) -> SpeechRequest {
        self.progress.timing.synthesis_started.get_or_insert(now_ms);
        SpeechRequest {
            text: self.chunks.get(index).cloned().unwrap_or_default(),
            output_id: self.request.output_id(index),
            voice: self.request.voice.clone(),
            emotion: self.request.emotion.clone(),
            model: self.request.model.clone(),
            ..SpeechRequest::default()
        }
    }

    pub(super) fn record_synthesis(&mut self, index: usize, speech: GeneratedSpeech, now_ms: u64) {
        self.progress.sources.push(speech.source);
        self.progress.timing.synthesized.get_or_insert(now_ms);
        self.progress.ready.insert(index, speech);
    }

    pub(super) fn begin_playing(&mut self, now_ms: u64, playbacks: &[RunningPlayback]) {
        self.progress.timing.playing.get_or_insert(now_ms);
        self.progress.stops = playbacks
            .iter()
            .map(|playback| std::sync::Arc::clone(&playback.stop))
            .collect();
    }

    pub(super) fn stop_all(&self) {
        self.progress.stops.iter().for_each(|stop| stop());
    }

    pub(super) fn override_failure(&mut self, message: Option<String>) {
        self.progress.failure = message.or(self.progress.failure.take());
    }
}
