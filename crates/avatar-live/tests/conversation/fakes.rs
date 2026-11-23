use std::cell::{Cell, RefCell};
use std::rc::Rc;

use avatar_live::boundaries::{Console, Speaker as SpeakerBoundary, TurnLog};
use avatar_live::spoken_turns::SpokenTurn;
use serde_json::{Value, json};

#[derive(Clone, Default)]
pub(super) struct Recorder {
    pub(super) log: Rc<RefCell<Vec<Value>>>,
    pub(super) printed: Rc<RefCell<Vec<String>>>,
    pub(super) buffered: Rc<Cell<f64>>,
}

impl Recorder {
    pub(super) fn push(&self, entry: Value) {
        self.log.borrow_mut().push(entry);
    }
}

pub(super) struct Speaker(pub(super) Recorder);
pub(super) struct TurnRecord(pub(super) Recorder);
pub(super) struct Screen(pub(super) Recorder);

impl SpeakerBoundary for Speaker {
    fn play(&mut self, pcm: &[u8]) {
        self.0.push(json!(["play", pcm]));
    }

    fn buffered_seconds(&self) -> f64 {
        self.0.buffered.get()
    }

    fn clear(&mut self) {
        self.0.push(json!(["clear"]));
    }
}

impl TurnLog for TurnRecord {
    fn append(&mut self, turn: &SpokenTurn) {
        self.0
            .push(json!(["append", turn.party.to_string(), turn.text]));
    }
}

impl Console for Screen {
    fn print(&mut self, line: &str) {
        self.0.printed.borrow_mut().push(line.to_owned());
    }
}
