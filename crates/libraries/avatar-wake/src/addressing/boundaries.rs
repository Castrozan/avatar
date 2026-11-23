use avatar_floor::addressee::Addressee;
use avatar_floor::quiet::Seat;
use avatar_routing::names::session::Session;
use serde_json::Value;

use super::decisions::DecisionUnavailable;

pub trait SessionSource {
    fn known_sessions(&self) -> Vec<Session>;
    fn refreshed_sessions(&self) -> Vec<Session>;
}

pub trait AddressingBoundaries {
    fn addressee(&self) -> Option<Addressee>;
    fn open_seat(&self) -> Option<Seat>;
    fn now(&self) -> f64;
    fn decide(&self, input: &Value, questions: &Value) -> Result<Value, DecisionUnavailable>;
    fn report_text(&self, text: &str);
    fn report_record(&self, record: &Value);
}
