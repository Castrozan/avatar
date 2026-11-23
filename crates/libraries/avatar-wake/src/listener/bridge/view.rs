use avatar_floor::addressee::Addressee;
use avatar_floor::quiet::Seat;
use avatar_routing::names::session::Session;
use serde_json::Value;

use super::View;
use crate::addressing::{AddressingBoundaries, DecisionUnavailable, SessionSource};
use crate::arbitration::room_holding::HoldingBoundaries;
use crate::listener::records::report_addressing;

impl SessionSource for View<'_> {
    fn known_sessions(&self) -> Vec<Session> {
        self.known()
    }

    fn refreshed_sessions(&self) -> Vec<Session> {
        self.refresh()
    }
}

impl AddressingBoundaries for View<'_> {
    fn addressee(&self) -> Option<Addressee> {
        self.boundaries.seat.addressee().ok().flatten()
    }

    fn open_seat(&self) -> Option<Seat> {
        self.boundaries.effects.open_seat()
    }

    fn now(&self) -> f64 {
        self.boundaries.effects.now()
    }

    fn decide(&self, input: &Value, questions: &Value) -> Result<Value, DecisionUnavailable> {
        self.boundaries.effects.decide(input, questions)
    }

    fn report_text(&self, text: &str) {
        self.boundaries.effects.report(text);
    }

    fn report_record(&self, record: &Value) {
        report_addressing(self.boundaries.effects, record);
    }
}

impl HoldingBoundaries for View<'_> {
    fn discord_call_host(&self) -> Option<String> {
        self.boundaries.effects.discord_call_host()
    }

    fn room_that_heard(&self, seconds: f64) -> Option<String> {
        self.boundaries.effects.room_that_heard(seconds)
    }

    fn now(&self) -> f64 {
        self.boundaries.effects.now()
    }

    fn claim_wake(&self, session: &Session) -> bool {
        self.boundaries.effects.claim_wake(session)
    }

    fn report(&self, text: &str) {
        self.boundaries.effects.report(text);
    }
}
