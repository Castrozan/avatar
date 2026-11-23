use std::cell::{Cell, RefCell};
use std::collections::VecDeque;

use avatar_floor::addressee::Addressee;
use avatar_floor::quiet::Seat;
use avatar_routing::names::session::Session;
use avatar_wake::addressing::{AddressingBoundaries, DecisionUnavailable, SessionSource};
use serde_json::Value;

pub(super) type Answer = Result<Value, DecisionUnavailable>;

#[derive(Default)]
pub(super) struct FakeAddressing {
    pub(super) holder: Option<Addressee>,
    pub(super) seat: Option<Seat>,
    pub(super) clock: Vec<f64>,
    pub(super) clock_index: Cell<usize>,
    pub(super) answers: RefCell<VecDeque<Answer>>,
    pub(super) reports: RefCell<Vec<Value>>,
    pub(super) decide_calls: RefCell<Vec<(Value, Value)>>,
    pub(super) known: Vec<Session>,
    pub(super) refreshed: Vec<Session>,
}

impl AddressingBoundaries for FakeAddressing {
    fn addressee(&self) -> Option<Addressee> {
        self.holder.clone()
    }

    fn open_seat(&self) -> Option<Seat> {
        self.seat
    }

    fn now(&self) -> f64 {
        let index = self.clock_index.get();
        self.clock_index.set(index + 1);
        self.clock[index.min(self.clock.len() - 1)]
    }

    fn decide(&self, input: &Value, questions: &Value) -> Answer {
        self.decide_calls
            .borrow_mut()
            .push((input.clone(), questions.clone()));
        self.answers
            .borrow_mut()
            .pop_front()
            .unwrap_or_else(missing_answer)
    }

    fn report_text(&self, text: &str) {
        self.reports
            .borrow_mut()
            .push(serde_json::json!({"text": text}));
    }

    fn report_record(&self, record: &Value) {
        self.reports.borrow_mut().push(record.clone());
    }
}

impl SessionSource for FakeAddressing {
    fn known_sessions(&self) -> Vec<Session> {
        self.known.clone()
    }

    fn refreshed_sessions(&self) -> Vec<Session> {
        self.refreshed.clone()
    }
}

fn missing_answer() -> Answer {
    Err(DecisionUnavailable {
        reason: "no scripted answer".to_owned(),
    })
}
