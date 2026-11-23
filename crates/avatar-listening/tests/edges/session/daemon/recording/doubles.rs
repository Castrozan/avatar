use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use avatar_listening::listener::daemon::{Timing, TranscribeFailure, Transcriber};
use avatar_listening::listener::diagnostics::Diagnostics;
use avatar_listening::listener::speaker_identity::{Heard, SpeakerIdentification};

pub(super) type Shared<Item> = Rc<RefCell<Vec<Item>>>;

pub(super) struct Script {
    pub(super) outcomes: VecDeque<Result<String, TranscribeFailure>>,
    pub(super) calls: Shared<(usize, Option<String>)>,
}

impl Script {
    pub(super) fn new(
        outcomes: VecDeque<Result<String, TranscribeFailure>>,
        calls: &Shared<(usize, Option<String>)>,
    ) -> Self {
        Self {
            outcomes,
            calls: Rc::clone(calls),
        }
    }
}

impl Transcriber for Script {
    fn transcribe(
        &mut self,
        segment: &[u8],
        language: Option<&str>,
    ) -> Result<String, TranscribeFailure> {
        self.calls
            .borrow_mut()
            .push((segment.len(), language.map(str::to_owned)));
        self.outcomes
            .pop_front()
            .unwrap_or_else(|| Err(TranscribeFailure::Failed("script exhausted".into())))
    }
}

pub(super) struct Identities(pub(super) VecDeque<Heard>);

impl SpeakerIdentification for Identities {
    fn identify(&mut self, _segment: &[u8], _notes: &mut dyn Diagnostics) -> Heard {
        self.0.pop_front().unwrap_or_default()
    }
}

pub(super) struct Ticks {
    pub(super) count: u32,
    pub(super) pauses: Shared<f64>,
}

impl Ticks {
    pub(super) fn new(pauses: &Shared<f64>) -> Self {
        Self {
            count: 0,
            pauses: Rc::clone(pauses),
        }
    }
}

impl Timing for Ticks {
    fn wall_seconds(&mut self) -> f64 {
        self.count += 1;
        1_700_000_000.0 + 0.25 * f64::from(self.count - 1)
    }

    fn pause(&mut self, seconds: f64) {
        self.pauses.borrow_mut().push(seconds);
    }
}
