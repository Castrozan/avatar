mod events;
mod transitions;

pub use events::{SequenceEffect, SequenceEvent};

use events::Prefetch;

#[derive(Debug)]
pub struct SpeechChunkSequence {
    chunk_count: usize,
    playing: Option<usize>,
    playback_done: bool,
    prefetched: Option<Prefetch>,
    stopped: bool,
    finished: bool,
}

impl SpeechChunkSequence {
    #[must_use]
    pub fn begin(chunk_count: usize) -> (Self, Vec<SequenceEffect>) {
        let sequence = Self {
            chunk_count,
            playing: None,
            playback_done: false,
            prefetched: None,
            stopped: false,
            finished: false,
        };
        (sequence, vec![SequenceEffect::Synthesize(0)])
    }

    #[must_use]
    pub fn handle(&mut self, event: SequenceEvent) -> Vec<SequenceEffect> {
        match event {
            SequenceEvent::Stop => self.stop(),
            _ if self.finished => Vec::new(),
            SequenceEvent::FirstChunkReady { holds_floor } => self.first_chunk_ready(holds_floor),
            SequenceEvent::ChunkReady(index) => self.chunk_ready(index),
            SequenceEvent::ChunkFailed => self.chunk_failed(),
            SequenceEvent::ChunkPlayed(index) => self.chunk_played(index),
        }
    }

    fn first_chunk_ready(&mut self, holds_floor: bool) -> Vec<SequenceEffect> {
        match (holds_floor, self.stopped) {
            (true, false) => self.begin_playing(0),
            _ => self.finish(),
        }
    }

    fn chunk_ready(&mut self, index: usize) -> Vec<SequenceEffect> {
        match self.playback_done.then_some(index) {
            Some(next) => self.begin_playing(next),
            None => self.prefetch(Prefetch::Ready(index)),
        }
    }

    fn chunk_failed(&mut self) -> Vec<SequenceEffect> {
        match (self.stopped, self.playing.is_some(), self.playback_done) {
            (true, _, _) => Vec::new(),
            (false, false, _) | (false, true, true) => self.fail(),
            (false, true, false) => self.prefetch(Prefetch::Failed),
        }
    }
}
