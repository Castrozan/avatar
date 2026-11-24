use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use super::machine::{Input, Runtime, run};
use super::services::SpeechServices;
use crate::server::{SpeechEvent, SpeechPort};
use crate::session::Handoff;
use crate::synthesis::contract::{CharacterSelection, SpeechGenerator};

pub struct SpeakingPort {
    known_voices: Vec<String>,
    inputs: UnboundedSender<Input>,
    events: Option<UnboundedReceiver<SpeechEvent>>,
}

impl SpeechPort for SpeakingPort {
    fn known_voices(&self) -> Vec<String> {
        self.known_voices.clone()
    }

    fn handle(&mut self, handoff: Handoff) {
        drop(self.inputs.send(Input::Handoff(handoff)));
    }

    fn take_events(&mut self) -> Option<UnboundedReceiver<SpeechEvent>> {
        self.events.take()
    }
}

pub fn start_speech_runtime<Generator: SpeechGenerator + CharacterSelection + 'static>(
    generator: Generator,
    services: SpeechServices,
) -> SpeakingPort {
    let known_voices = generator.character_names();
    let (inputs, receiver) = mpsc::unbounded_channel();
    let (outputs, events) = mpsc::unbounded_channel();
    let runtime = Runtime::new(generator, services, inputs.clone(), outputs);
    tokio::spawn(run(runtime, receiver));
    SpeakingPort {
        known_voices,
        inputs,
        events: Some(events),
    }
}
