use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};
use std::time::Instant;

use avatar_room_audio::mouth::{MouthLevels, VoiceLevelSink};

use super::queue::ChunkQueue;
use super::sink::SeatSink;
use super::{LEAD_SECONDS, playing_seconds};
use crate::failure::Failure;

pub type Hush = Box<dyn Fn() + Send + Sync>;

pub(super) struct Mouth<Voice: VoiceLevelSink> {
    pub(super) levels: Mutex<MouthLevels<Voice>>,
    pub(super) arrived: Condvar,
}

#[derive(Default)]
pub(super) struct Status {
    pub(super) stopped: AtomicBool,
    pub(super) failure: Mutex<Option<String>>,
}

pub(super) struct Shared<Voice: VoiceLevelSink> {
    pub(super) queue: ChunkQueue,
    pub(super) sink: Mutex<Box<dyn SeatSink>>,
    pub(super) hush: Hush,
    pub(super) mouth: Mouth<Voice>,
    pub(super) status: Status,
    pub(super) origin: Instant,
    pub(super) rate: u32,
}

pub(super) fn held<Value>(mutex: &Mutex<Value>) -> MutexGuard<'_, Value> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

pub(super) fn now_of<Voice: VoiceLevelSink>(shared: &Shared<Voice>) -> f64 {
    shared.origin.elapsed().as_secs_f64()
}

pub(super) fn fail<Voice: VoiceLevelSink>(shared: &Shared<Voice>, message: &str) {
    *held(&shared.status.failure) = Some(message.to_owned());
}

pub(super) fn running<Voice: VoiceLevelSink>(shared: &Shared<Voice>) -> bool {
    !shared.status.stopped.load(Ordering::SeqCst)
}

pub(super) fn show<Voice: VoiceLevelSink>(shared: &Shared<Voice>, chunk: &[u8]) {
    held(&shared.mouth.levels).show(chunk, now_of(shared));
    shared.mouth.arrived.notify_all();
}

pub(super) fn deliver<Voice: VoiceLevelSink>(
    shared: &Shared<Voice>,
    turn: (u64, &[u8]),
) -> Result<(), Failure> {
    held(&shared.sink).open()?;
    shared.queue.wait_turn(|| now_of(shared), LEAD_SECONDS);
    let heard_from = (now_of(shared), playing_seconds(turn.1.len(), shared.rate));
    let sent = shared
        .queue
        .commit(turn.0, heard_from, || held(&shared.sink).send(turn.1))?;
    sent.then(|| show(shared, turn.1));
    Ok(())
}

pub(super) fn feed_step<Voice: VoiceLevelSink>(shared: &Shared<Voice>) -> bool {
    let Some((generation, chunk)) = shared.queue.take_next() else {
        return false;
    };
    let delivered = deliver(shared, (generation, &chunk));
    delivered
        .map_err(|failure| fail(shared, &failure.message))
        .is_ok()
}

pub(super) fn feed<Voice: VoiceLevelSink>(shared: &Shared<Voice>) {
    while feed_step(shared) {}
    held(&shared.sink).close();
}

pub struct SpeakerParts<Voice> {
    pub sink: Box<dyn SeatSink>,
    pub hush: Hush,
    pub voice: Voice,
    pub rate: u32,
}

impl<Voice: VoiceLevelSink> Mouth<Voice> {
    fn new(voice: Voice) -> Self {
        Self {
            levels: Mutex::new(MouthLevels::new(voice, 0.0)),
            arrived: Condvar::new(),
        }
    }
}

pub(super) fn shared_of<Voice: VoiceLevelSink>(parts: SpeakerParts<Voice>) -> Shared<Voice> {
    Shared {
        queue: ChunkQueue::default(),
        sink: Mutex::new(parts.sink),
        hush: parts.hush,
        mouth: Mouth::new(parts.voice),
        status: Status::default(),
        origin: Instant::now(),
        rate: parts.rate,
    }
}
