use avatar_floor::addressee::Addressee;
use avatar_floor::quiet::{Entry, ListenerPurpose, Seat};
use avatar_routing::names::session::Session;
use serde_json::Value;

use super::failure::Boundary;
use crate::addressing::DecisionUnavailable;

pub trait WakeEffects {
    fn sessions(&self) -> Vec<Session>;
    fn discord_call_host(&self) -> Option<String>;
    fn open_seat(&self) -> Option<Seat>;
    fn discord_call_ended(&self) -> bool;
    fn transcribe(&self, segment: &[u8]) -> Boundary<String>;
    fn decide(&self, input: &Value, questions: &Value) -> Result<Value, DecisionUnavailable>;
    fn room_that_heard(&self, seconds: f64) -> Option<String>;
    fn claim_wake(&self, session: &Session) -> bool;
    fn prompt_now(&self, pane: &str, text: &str, host: Option<&str>) -> Boundary<bool>;
    fn notify_at_idle(&self, pane: &str, text: &str, host: Option<&str>) -> Boundary<()>;
    fn close_conversation(&self);
    fn report(&self, line: &str);
    fn room_name(&self) -> Option<String>;
    fn timestamp(&self) -> String;
    fn now(&self) -> f64;
    fn monotonic(&self) -> f64;
    fn sleep(&self, seconds: f64);
    fn pause(&self, seconds: f64);
}

pub trait ConversationSeat {
    fn conversation_listening(&self) -> bool;
    fn open_conversation(&self) -> Boundary<bool>;
    fn transcript_entries(&self) -> Boundary<Vec<Entry>>;
    fn speech_in_progress(&self) -> bool;
    fn listening_purpose(&self) -> Boundary<Option<ListenerPurpose>>;
    fn addressee(&self) -> Boundary<Option<Addressee>>;
    fn quiet_seconds(&self) -> f64;
    fn hand_to(&self, addressee: &Addressee) -> Boundary<()>;
}

pub trait Microphone {
    fn start(&mut self);
    fn next_frame(&mut self) -> Option<Vec<u8>>;
    fn stop(&mut self);
}

#[derive(Clone, Copy)]
pub struct Boundaries<'a> {
    pub effects: &'a dyn WakeEffects,
    pub seat: &'a dyn ConversationSeat,
}
