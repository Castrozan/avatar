use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use super::machine::{Input, Runtime, run};
use super::services::SpeechServices;
use crate::server::{SpeechEvent, SpeechPort};
use crate::session::Handoff;
use crate::synthesis::contract::SpeechGenerator;

pub struct SpeakingPort {
    inputs: UnboundedSender<Input>,
    events: Option<UnboundedReceiver<SpeechEvent>>,
}

impl SpeechPort for SpeakingPort {
    fn handle(&mut self, handoff: Handoff) {
        drop(self.inputs.send(Input::Handoff(handoff)));
    }

    fn take_events(&mut self) -> Option<UnboundedReceiver<SpeechEvent>> {
        self.events.take()
    }
}

pub fn start_speech_runtime<Generator: SpeechGenerator + 'static>(
    generator: Generator,
    services: SpeechServices,
) -> SpeakingPort {
    let (inputs, receiver) = mpsc::unbounded_channel();
    let (outputs, events) = mpsc::unbounded_channel();
    let runtime = Runtime::new(generator, services, inputs.clone(), outputs);
    tokio::spawn(run(runtime, receiver));
    SpeakingPort {
        inputs,
        events: Some(events),
    }
}
