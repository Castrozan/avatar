use std::cell::{Cell, RefCell};

use serde::Deserialize;
use serde_json::{Value, json};

use crate::quiet::{self, ListenerPurpose, QuietBoundaries, Seat};

#[derive(Deserialize)]
struct QuietClose {
    quiet_seconds: f64,
    purpose: ListenerPurpose,
    dry_run: bool,
}

#[derive(Deserialize)]
struct CallClose {
    seat: Seat,
    call_ended: bool,
    dry_run: bool,
}

struct Recording {
    quiet_seconds: f64,
    purpose: ListenerPurpose,
    seat: Seat,
    call_ended: bool,
    reports: RefCell<Vec<String>>,
    close_calls: Cell<usize>,
}

impl QuietBoundaries for Recording {
    fn quiet_seconds(&self) -> f64 {
        self.quiet_seconds
    }

    fn listening_purpose(&self) -> ListenerPurpose {
        self.purpose
    }

    fn open_seat(&self) -> Seat {
        self.seat
    }

    fn discord_call_ended(&self) -> bool {
        self.call_ended
    }

    fn report(&self, message: &str) {
        self.reports.borrow_mut().push(message.to_owned());
    }

    fn close_conversation(&self) {
        self.close_calls.set(self.close_calls.get() + 1);
    }
}

impl Recording {
    fn new(quiet_seconds: f64, purpose: ListenerPurpose, seat: Seat, call_ended: bool) -> Self {
        Self {
            quiet_seconds,
            purpose,
            seat,
            call_ended,
            reports: RefCell::default(),
            close_calls: Cell::default(),
        }
    }

    fn outcome(self, closed: bool) -> Value {
        json!({
            "closed": closed,
            "reports": self.reports.into_inner(),
            "close_calls": self.close_calls.get(),
        })
    }
}

pub(super) fn close_when_quiet(request: &Value) -> Result<Value, String> {
    let close = QuietClose::deserialize(request).map_err(|error| error.to_string())?;
    let recording = Recording::new(close.quiet_seconds, close.purpose, Seat::Room, false);
    let closed = quiet::close_when_quiet(&recording, close.dry_run);
    Ok(recording.outcome(closed))
}

pub(super) fn close_when_the_call_ended(request: &Value) -> Result<Value, String> {
    let close = CallClose::deserialize(request).map_err(|error| error.to_string())?;
    let recording = Recording::new(
        0.0,
        ListenerPurpose::Conversation,
        close.seat,
        close.call_ended,
    );
    let closed = quiet::close_when_the_call_ended(&recording, close.dry_run);
    Ok(recording.outcome(closed))
}
