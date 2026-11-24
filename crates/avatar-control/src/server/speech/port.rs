use tokio::sync::mpsc::UnboundedReceiver;

use crate::session::{Handoff, Outbound};

#[derive(Debug, Clone, PartialEq)]
pub enum SpeechEvent {
    Deliver(Outbound),
    Activity(bool),
    Phase(String),
}

pub trait SpeechPort: Send + 'static {
    fn known_voices(&self) -> Vec<String>;

    fn handle(&mut self, handoff: Handoff);

    fn take_events(&mut self) -> Option<UnboundedReceiver<SpeechEvent>>;
}

#[derive(Debug, Default)]
pub struct DiscardingSpeechPort;

impl SpeechPort for DiscardingSpeechPort {
    fn known_voices(&self) -> Vec<String> {
        Vec::new()
    }

    fn handle(&mut self, _handoff: Handoff) {}

    fn take_events(&mut self) -> Option<UnboundedReceiver<SpeechEvent>> {
        None
    }
}
