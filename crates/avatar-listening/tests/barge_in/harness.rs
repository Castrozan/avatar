use std::cell::Cell;
use std::rc::Rc;

use avatar_listening::barge_in::decision::{BargeIn, Reply, VoiceActivity};
use avatar_listening::pcm::frame_level;
use avatar_listening::speech_segmenter::SPEECH_LEVEL;

const FRAME_SAMPLES: usize = 512;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Speech,
    Noise,
    Quiet,
    Silence,
}

pub(crate) fn kind_named(name: &str) -> Result<Kind, Box<dyn std::error::Error>> {
    match name {
        "speech" => Ok(Kind::Speech),
        "noise" => Ok(Kind::Noise),
        "quiet" => Ok(Kind::Quiet),
        "silence" => Ok(Kind::Silence),
        _ => Err("unknown frame kind".into()),
    }
}

fn sample_of(kind: Kind) -> i16 {
    match kind {
        Kind::Speech => 3000,
        Kind::Noise => 3001,
        Kind::Quiet => 100,
        Kind::Silence => 0,
    }
}

pub(crate) fn frame_of(kind: Kind) -> Vec<u8> {
    (0..FRAME_SAMPLES)
        .flat_map(|_| sample_of(kind).to_le_bytes())
        .collect()
}

pub(crate) fn is_loud(frame: &[u8]) -> bool {
    frame_level(frame) >= SPEECH_LEVEL
}

pub(crate) struct Cells {
    pub(crate) speaking: Rc<Cell<bool>>,
    pub(crate) stops: Rc<Cell<usize>>,
    pub(crate) judged: Rc<Cell<usize>>,
    pub(crate) resets: Rc<Cell<usize>>,
}

impl Cells {
    fn new() -> Self {
        Self {
            speaking: Rc::new(Cell::new(true)),
            stops: Rc::new(Cell::new(0)),
            judged: Rc::new(Cell::new(0)),
            resets: Rc::new(Cell::new(0)),
        }
    }
}

pub(crate) struct SharedReply(Rc<Cell<bool>>, Rc<Cell<usize>>);

impl Reply for SharedReply {
    fn speaking(&self) -> bool {
        self.0.get()
    }

    fn stop(&mut self) {
        self.1.set(self.1.get() + 1);
    }
}

pub(crate) struct SharedVoice(Rc<Cell<usize>>, Rc<Cell<usize>>);

impl VoiceActivity for SharedVoice {
    fn reset(&mut self) {
        self.1.set(self.1.get() + 1);
    }

    fn speech_probability(&mut self, frame: &[u8]) -> f64 {
        self.0.set(self.0.get() + 1);
        probability_of(i16::from_le_bytes([frame[0], frame[1]]))
    }
}

fn probability_of(sample: i16) -> f64 {
    match sample {
        3000 | 100 => 0.97,
        _ => 0.05,
    }
}

pub(crate) struct Harness {
    pub(crate) cells: Cells,
    pub(crate) barge: BargeIn<SharedReply, SharedVoice>,
}

impl Harness {
    pub(crate) fn start() -> Self {
        let cells = Cells::new();
        let reply = SharedReply(Rc::clone(&cells.speaking), Rc::clone(&cells.stops));
        let voice = SharedVoice(Rc::clone(&cells.judged), Rc::clone(&cells.resets));
        Self {
            cells,
            barge: BargeIn::new(reply, voice),
        }
    }

    pub(crate) fn hear(&mut self, frame: &[u8]) -> bool {
        self.barge.hear(frame, is_loud(frame)).is_some()
    }
}
