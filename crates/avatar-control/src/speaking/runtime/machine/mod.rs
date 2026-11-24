mod floor_effects;
mod input;
mod intake;
mod phase;
mod steps;
mod turn;

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use super::services::SpeechServices;
use crate::server::SpeechEvent;
use crate::session::Handoff;
use crate::speech::floor::{FloorEffect, SpeechFloor, TurnId};
use crate::synthesis::contract::{CharacterSelection, SpeechGenerator};

use turn::Turn;

pub(super) use input::Input;

#[derive(Default)]
struct State {
    floor: SpeechFloor,
    turns: HashMap<TurnId, Turn>,
    deferred: HashMap<TurnId, VecDeque<FloorEffect>>,
    next_turn: TurnId,
}

pub(super) struct Runtime<Generator> {
    generator: Arc<Generator>,
    services: SpeechServices,
    inputs: UnboundedSender<Input>,
    outputs: UnboundedSender<SpeechEvent>,
    state: State,
    phase: &'static str,
}

impl<Generator: SpeechGenerator + 'static> Runtime<Generator> {
    pub(super) fn new(
        generator: Generator,
        services: SpeechServices,
        inputs: UnboundedSender<Input>,
        outputs: UnboundedSender<SpeechEvent>,
    ) -> Self {
        Self {
            generator: Arc::new(generator),
            services,
            inputs,
            outputs,
            state: State::default(),
            phase: phase::IDLE,
        }
    }

    fn emit(&self, event: SpeechEvent) {
        drop(self.outputs.send(event));
    }
}

impl<Generator: SpeechGenerator + CharacterSelection + 'static> Runtime<Generator> {
    fn handle(&mut self, input: Input) {
        match input {
            Input::Handoff(handoff) => self.hand_off(handoff),
            Input::ChannelOpened(opened) => self.channel_opened(opened),
            Input::ChannelReleased(released) => self.channel_released(released),
            Input::ChunkSynthesized(synthesized) => self.chunk_synthesized(synthesized),
            Input::ChunkPlayed(played) => self.chunk_played(&played),
        }
        self.publish_phase();
    }

    fn hand_off(&mut self, handoff: Handoff) {
        match handoff {
            Handoff::Speak {
                client,
                received_ms,
                command,
            } => self.speak(client, received_ms, &command),
            Handoff::StopSpeaking => self.interrupt(),
            Handoff::SelectVoice(name) => drop(self.generator.select_character(&name)),
        }
    }
}

async fn next_input(
    inputs: &mut UnboundedReceiver<Input>,
    outputs: &UnboundedSender<SpeechEvent>,
) -> Option<Input> {
    tokio::select! {
        input = inputs.recv() => input,
        () = outputs.closed() => None,
    }
}

pub(super) async fn run<Generator: SpeechGenerator + CharacterSelection + 'static>(
    mut runtime: Runtime<Generator>,
    mut inputs: UnboundedReceiver<Input>,
) {
    while let Some(input) = next_input(&mut inputs, &runtime.outputs).await {
        runtime.handle(input);
    }
}
