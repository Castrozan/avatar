use std::cell::RefCell;

use avatar_floor::quiet::{ListenerPurpose, QuietBoundaries, Seat};

pub(crate) struct FakeBoundaries {
    pub(crate) quiet_for_seconds: f64,
    pub(crate) purpose: ListenerPurpose,
    pub(crate) seat: Seat,
    pub(crate) call_ended: bool,
    pub(crate) calls: RefCell<Vec<String>>,
}

impl FakeBoundaries {
    pub(crate) fn new(
        quiet_for_seconds: f64,
        purpose: ListenerPurpose,
        seat: Seat,
        call_ended: bool,
    ) -> Self {
        Self {
            quiet_for_seconds,
            purpose,
            seat,
            call_ended,
            calls: RefCell::new(Vec::new()),
        }
    }

    pub(crate) fn recorded(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        Ok(self.calls.try_borrow()?.clone())
    }

    fn record(&self, call: &str) {
        self.calls.borrow_mut().push(call.to_owned());
    }
}

impl QuietBoundaries for FakeBoundaries {
    fn quiet_seconds(&self) -> f64 {
        self.record("quiet_seconds");
        self.quiet_for_seconds
    }

    fn listening_purpose(&self) -> ListenerPurpose {
        self.record("listening_purpose");
        self.purpose
    }

    fn open_seat(&self) -> Seat {
        self.record("open_seat");
        self.seat
    }

    fn discord_call_ended(&self) -> bool {
        self.record("discord_call_ended");
        self.call_ended
    }

    fn report(&self, message: &str) {
        self.record(&format!("report:{message}"));
    }

    fn close_conversation(&self) {
        self.record("close_conversation");
    }
}
