#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SequenceEvent {
    FirstChunkReady { holds_floor: bool },
    ChunkReady(usize),
    ChunkPlayed(usize),
    Stop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SequenceEffect {
    Synthesize(usize),
    Play(usize),
    StopPlayback,
    Finished,
}

#[derive(Debug)]
pub struct SpeechChunkSequence {
    chunk_count: usize,
    playing: Option<usize>,
    playback_done: bool,
    prefetched: Option<usize>,
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
            None => self.prefetch(index),
        }
    }

    fn prefetch(&mut self, index: usize) -> Vec<SequenceEffect> {
        self.prefetched = Some(index);
        Vec::new()
    }

    fn chunk_played(&mut self, index: usize) -> Vec<SequenceEffect> {
        match (self.stopped, index + 1 < self.chunk_count) {
            (false, true) => self.play_prefetched(),
            _ => self.finish(),
        }
    }

    fn play_prefetched(&mut self) -> Vec<SequenceEffect> {
        self.playback_done = true;
        self.prefetched
            .take()
            .map_or_else(Vec::new, |next| self.begin_playing(next))
    }

    fn stop(&mut self) -> Vec<SequenceEffect> {
        self.stopped = true;
        let playback = self
            .playing
            .is_some()
            .then_some(SequenceEffect::StopPlayback);
        let completion = (self.playback_done && !self.finished).then(|| self.finish());
        playback
            .into_iter()
            .chain(completion.unwrap_or_default())
            .collect()
    }

    fn begin_playing(&mut self, index: usize) -> Vec<SequenceEffect> {
        self.playing = Some(index);
        self.playback_done = false;
        let prefetch =
            (index + 1 < self.chunk_count).then(|| SequenceEffect::Synthesize(index + 1));
        std::iter::once(SequenceEffect::Play(index))
            .chain(prefetch)
            .collect()
    }

    fn finish(&mut self) -> Vec<SequenceEffect> {
        self.finished = true;
        vec![SequenceEffect::Finished]
    }
}
