use super::SpeechChunkSequence;
use super::events::{Prefetch, SequenceEffect};

impl SpeechChunkSequence {
    pub(super) fn prefetch(&mut self, prefetch: Prefetch) -> Vec<SequenceEffect> {
        self.prefetched = Some(prefetch);
        Vec::new()
    }

    pub(super) fn chunk_played(&mut self, index: usize) -> Vec<SequenceEffect> {
        match (self.stopped, index + 1 < self.chunk_count) {
            (false, true) => self.play_prefetched(),
            _ => self.finish(),
        }
    }

    fn play_prefetched(&mut self) -> Vec<SequenceEffect> {
        self.playback_done = true;
        match self.prefetched.take() {
            None => Vec::new(),
            Some(Prefetch::Ready(next)) => self.begin_playing(next),
            Some(Prefetch::Failed) => self.fail(),
        }
    }

    pub(super) fn stop(&mut self) -> Vec<SequenceEffect> {
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

    pub(super) fn begin_playing(&mut self, index: usize) -> Vec<SequenceEffect> {
        self.playing = Some(index);
        self.playback_done = false;
        let prefetch =
            (index + 1 < self.chunk_count).then(|| SequenceEffect::Synthesize(index + 1));
        std::iter::once(SequenceEffect::Play(index))
            .chain(prefetch)
            .collect()
    }

    pub(super) fn finish(&mut self) -> Vec<SequenceEffect> {
        self.finished = true;
        vec![SequenceEffect::Finished]
    }

    pub(super) fn fail(&mut self) -> Vec<SequenceEffect> {
        self.finished = true;
        vec![SequenceEffect::Failed]
    }
}
