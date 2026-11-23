use std::future::Future;
use std::path::Path;

use futures_util::future::join_all;
use serde_json::Value;
use tokio::sync::mpsc::UnboundedSender;

use crate::speaking::runtime::machine::input::ChunkPlayed;
use crate::speaking::runtime::machine::turn::Turn;
use crate::speaking::runtime::machine::{Input, Runtime};
use crate::speaking::runtime::players::{PlaybackTarget, RunningPlayback};
use crate::speaking::speaking_started;
use crate::speech::chunk_sequence::SequenceEvent;
use crate::speech::floor::TurnId;
use crate::synthesis::contract::{GeneratedSpeech, SpeechGenerator};

struct ChunkPlayback {
    speech: GeneratedSpeech,
    announcement: Value,
    targets: Vec<PlaybackTarget>,
}

async fn report_played(
    finished: impl Future<Output = Vec<()>>,
    played: ChunkPlayed,
    inputs: UnboundedSender<Input>,
) {
    finished.await;
    drop(inputs.send(Input::ChunkPlayed(played)));
}

impl<Generator: SpeechGenerator + 'static> Runtime<Generator> {
    pub(crate) fn play(&mut self, turn: TurnId, index: usize) {
        self.take_chunk(turn, index)
            .into_iter()
            .for_each(|chunk| self.start_chunk(turn, index, chunk));
    }

    fn take_chunk(&mut self, turn: TurnId, index: usize) -> Option<ChunkPlayback> {
        let record = self.state.turns.get_mut(&turn)?;
        let speech = record.progress.ready.remove(&index)?;
        let text = record.chunks.get(index)?;
        Some(ChunkPlayback {
            announcement: speaking_started(&record.request, text, &speech.vowels),
            targets: record.targets.clone(),
            speech,
        })
    }

    fn start_chunk(&mut self, turn: TurnId, index: usize, chunk: ChunkPlayback) {
        self.emit_to_renderer(chunk.announcement);
        let playbacks = self.start_players(&chunk.targets, &chunk.speech.audio_path);
        self.record_playing(turn, &playbacks);
        self.await_playbacks(ChunkPlayed { turn, index }, playbacks);
    }

    fn start_players(&self, targets: &[PlaybackTarget], audio_path: &Path) -> Vec<RunningPlayback> {
        let players = &self.services.players;
        targets
            .iter()
            .map(|target| players.start(*target, audio_path))
            .collect()
    }

    fn record_playing(&mut self, turn: TurnId, playbacks: &[RunningPlayback]) {
        let now = self.services.clock.now_ms();
        self.state
            .turns
            .get_mut(&turn)
            .into_iter()
            .for_each(|record| record.begin_playing(now, playbacks));
    }

    fn await_playbacks(&self, played: ChunkPlayed, playbacks: Vec<RunningPlayback>) {
        let finished = join_all(playbacks.into_iter().map(|playback| playback.finished));
        tokio::spawn(report_played(finished, played, self.inputs.clone()));
    }

    pub(crate) fn stop_playback(&self, turn: TurnId) {
        self.state
            .turns
            .get(&turn)
            .into_iter()
            .for_each(Turn::stop_all);
    }

    pub(crate) fn chunk_played(&mut self, played: &ChunkPlayed) {
        self.sequence_event(played.turn, SequenceEvent::ChunkPlayed(played.index));
    }
}
