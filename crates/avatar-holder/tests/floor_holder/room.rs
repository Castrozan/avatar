use avatar_holder::default_holder::DefaultHolder;
use avatar_holder::floor_holder::{FloorClock, HoldingInterruption, RoomFloor};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

#[derive(Default)]
pub(crate) struct RoomState {
    pub(crate) addressed_now: bool,
    pub(crate) still_open_now: bool,
    pub(crate) consumer_free: bool,
    pub(crate) utterances: VecDeque<String>,
    pub(crate) addressed_changes: VecDeque<(usize, bool)>,
    pub(crate) pause_interruption: Option<HoldingInterruption>,
    pub(crate) taken_by: Vec<DefaultHolder>,
    pub(crate) pauses: Vec<f64>,
    pub(crate) clock_seconds: f64,
    pub(crate) still_open_checks: Vec<f64>,
    pub(crate) next_utterance_calls: usize,
}

#[derive(Clone, Default)]
pub(crate) struct SharedRoom(pub(crate) Rc<RefCell<RoomState>>);

fn apply_scripted_addressing(state: &mut RoomState) {
    let pauses = state.pauses.len();
    let due = state
        .addressed_changes
        .front()
        .copied()
        .filter(|change| change.0 == pauses);
    if let Some((_, value)) = due {
        state.addressed_now = value;
        state.addressed_changes.pop_front();
    }
}

fn next_scripted_utterance(state: &mut RoomState) -> Option<String> {
    state.next_utterance_calls += 1;
    let heard = state.utterances.pop_front();
    if heard.is_none() {
        state.addressed_now = false;
        state.still_open_now = false;
    }
    heard
}

impl RoomFloor for SharedRoom {
    fn addressed(&mut self) -> bool {
        self.0.borrow().addressed_now
    }

    fn still_open(&mut self) -> bool {
        let mut state = self.0.borrow_mut();
        let now = state.clock_seconds;
        state.still_open_checks.push(now);
        state.still_open_now
    }

    fn take_the_floor(&mut self, holder: &DefaultHolder) {
        self.0.borrow_mut().taken_by.push(holder.clone());
    }

    fn try_take_consumer(&mut self) -> bool {
        self.0.borrow().consumer_free
    }

    fn release_consumer(&mut self) {}

    fn next_utterance(&mut self) -> Result<Option<String>, HoldingInterruption> {
        Ok(next_scripted_utterance(&mut self.0.borrow_mut()))
    }
}

impl FloorClock for SharedRoom {
    fn now_seconds(&mut self) -> f64 {
        self.0.borrow().clock_seconds
    }

    fn pause(&mut self, seconds: f64) -> Result<(), HoldingInterruption> {
        let mut state = self.0.borrow_mut();
        state.pauses.push(seconds);
        state.clock_seconds += seconds;
        apply_scripted_addressing(&mut state);
        state.pause_interruption.map_or(Ok(()), Err)
    }
}
