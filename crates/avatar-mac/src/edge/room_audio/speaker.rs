use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::thread::{self, JoinHandle};

use avatar_live::boundaries::Speaker;
use avatar_room_audio::mouth::VoiceLevelSink;

use super::mouth_loop::deliver_mouth;
use super::shared::{Shared, SpeakerParts, feed, held, now_of, shared_of};
use super::{CHUNK_MILLISECONDS, chunk_bytes, seconds_of};

pub struct SeatSpeaker<Voice: VoiceLevelSink + Send + 'static> {
    shared: Arc<Shared<Voice>>,
    workers: Vec<JoinHandle<()>>,
    chunk_bytes: usize,
}

fn workers_of<Voice: VoiceLevelSink + Send + 'static>(
    shared: &Arc<Shared<Voice>>,
) -> Vec<JoinHandle<()>> {
    let (feeding, showing) = (Arc::clone(shared), Arc::clone(shared));
    vec![
        thread::spawn(move || feed(&feeding)),
        thread::spawn(move || deliver_mouth(&showing)),
    ]
}

impl<Voice: VoiceLevelSink + Send + 'static> SeatSpeaker<Voice> {
    #[must_use]
    pub fn start(parts: SpeakerParts<Voice>) -> Self {
        let chunk_bytes = chunk_bytes(parts.rate, CHUNK_MILLISECONDS);
        let shared = Arc::new(shared_of(parts));
        let workers = workers_of(&shared);
        Self {
            shared,
            workers,
            chunk_bytes,
        }
    }

    #[must_use]
    pub fn failure(&self) -> Option<String> {
        held(&self.shared.status.failure).clone()
    }

    pub fn close(&mut self) {
        self.shared.status.stopped.store(true, Ordering::SeqCst);
        self.shared.queue.stop();
        held(&self.shared.mouth.levels).close(now_of(&self.shared));
        self.shared.mouth.arrived.notify_all();
        self.workers.clear();
    }
}

impl<Voice: VoiceLevelSink + Send + 'static> Speaker for SeatSpeaker<Voice> {
    fn play(&mut self, pcm: &[u8]) {
        let chunks = pcm.chunks(self.chunk_bytes).map(<[u8]>::to_vec).collect();
        self.shared.queue.enqueue(chunks);
    }

    fn buffered_seconds(&self) -> f64 {
        seconds_of(self.shared.queue.buffered())
    }

    fn clear(&mut self) {
        self.shared.queue.clear();
        (self.shared.hush)();
        let _ = held(&self.shared.mouth.levels).silence(now_of(&self.shared));
    }
}
