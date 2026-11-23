use crate::session::Handoff;

pub trait SpeechPort: Send + 'static {
    fn handle(&mut self, handoff: Handoff);
}

#[derive(Debug, Default)]
pub struct DiscardingSpeechPort;

impl SpeechPort for DiscardingSpeechPort {
    fn handle(&mut self, _handoff: Handoff) {}
}
