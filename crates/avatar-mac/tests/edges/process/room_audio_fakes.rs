use avatar_mac::edge::room_audio::{SeatSink, SeatSpeaker, SpeakerParts};
use avatar_mac::failure::Failure;
use avatar_room_audio::mouth::VoiceLevelSink;
use std::io;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

pub(super) const RATE: u32 = 24000;

#[derive(Clone, Default)]
pub(super) struct Wire {
    pub(super) sent: Arc<Mutex<Vec<Vec<u8>>>>,
    pub(super) closed: Arc<AtomicBool>,
    pub(super) refuse: Arc<AtomicBool>,
}

impl Wire {
    pub(super) fn total(&self) -> Vec<u8> {
        let held = self.sent.lock().unwrap_or_else(PoisonError::into_inner);
        held.concat()
    }

    pub(super) fn longest(&self) -> usize {
        let held = self.sent.lock().unwrap_or_else(PoisonError::into_inner);
        held.iter().map(Vec::len).max().unwrap_or(0)
    }
}

pub(super) fn refuse_or_keep(wire: &Wire, pcm: &[u8]) -> Result<(), Failure> {
    if wire.refuse.load(Ordering::SeqCst) {
        return Err(Failure::runtime("the seat refused"));
    }
    let mut held = wire.sent.lock().unwrap_or_else(PoisonError::into_inner);
    held.push(pcm.to_vec());
    Ok(())
}

impl SeatSink for Wire {
    fn open(&mut self) -> Result<(), Failure> {
        Ok(())
    }

    fn send(&mut self, pcm: &[u8]) -> Result<(), Failure> {
        refuse_or_keep(self, pcm)
    }

    fn close(&mut self) {
        self.closed.store(true, Ordering::SeqCst);
    }
}

#[derive(Clone, Default)]
pub(super) struct Levels {
    pub(super) texts: Arc<Mutex<Vec<String>>>,
    pub(super) closed: Arc<AtomicBool>,
}

impl Levels {
    pub(super) fn values(&self) -> Vec<f64> {
        let held = self.texts.lock().unwrap_or_else(PoisonError::into_inner);
        let parsed = held
            .iter()
            .filter_map(|text| serde_json::from_str::<serde_json::Value>(text).ok());
        parsed
            .filter_map(|message| message["value"].as_f64())
            .collect()
    }
}

impl VoiceLevelSink for Levels {
    fn send_text(&mut self, text: &str) -> io::Result<()> {
        let mut held = self.texts.lock().unwrap_or_else(PoisonError::into_inner);
        held.push(text.to_owned());
        Ok(())
    }

    fn close(&mut self) {
        self.closed.store(true, Ordering::SeqCst);
    }
}

pub(super) fn eventually(condition: impl Fn() -> bool) -> bool {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !condition() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(10));
    }
    condition()
}

pub(super) fn speaker(
    wire: &Wire,
    levels: &Levels,
    hushes: &Arc<AtomicUsize>,
) -> SeatSpeaker<Levels> {
    let counted = Arc::clone(hushes);
    SeatSpeaker::start(SpeakerParts {
        sink: Box::new(wire.clone()),
        hush: Box::new(move || {
            let _ = counted.fetch_add(1, Ordering::SeqCst);
        }),
        voice: levels.clone(),
        rate: RATE,
    })
}
