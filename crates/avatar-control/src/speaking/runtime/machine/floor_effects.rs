use std::collections::VecDeque;

use super::Runtime;
use crate::speech::chunk_sequence::SequenceEvent;
use crate::speech::floor::{FloorEffect, FloorEvent, TurnId};
use crate::synthesis::contract::SpeechGenerator;

fn suspension(effect: &FloorEffect) -> Option<TurnId> {
    match effect {
        FloorEffect::ReleaseAgentChannel(turn) => Some(*turn),
        _ => None,
    }
}

impl<Generator: SpeechGenerator + 'static> Runtime<Generator> {
    pub(super) fn floor_event(&mut self, event: FloorEvent) {
        let effects = self.state.floor.handle(event).unwrap_or_default();
        self.execute(effects.into());
    }

    pub(super) fn resume(&mut self, turn: TurnId) {
        let effects = self.state.deferred.remove(&turn).unwrap_or_default();
        self.execute(effects);
    }

    fn execute(&mut self, mut effects: VecDeque<FloorEffect>) {
        effects
            .pop_front()
            .into_iter()
            .for_each(|effect| self.execute_from(effect, std::mem::take(&mut effects)));
    }

    fn execute_from(&mut self, effect: FloorEffect, rest: VecDeque<FloorEffect>) {
        self.perform(effect);
        match suspension(&effect) {
            Some(turn) => drop(self.state.deferred.insert(turn, rest)),
            None => self.execute(rest),
        }
    }

    fn perform(&mut self, effect: FloorEffect) {
        match effect {
            FloorEffect::OpenAgentChannel(turn) => self.open_channel(turn),
            FloorEffect::ReleaseAgentChannel(turn) => self.release_channel(turn),
            FloorEffect::StartSpeech(turn) => self.start_speech(turn),
            FloorEffect::StopPlayback(turn) => self.sequence_event(turn, SequenceEvent::Stop),
            FloorEffect::Settle { turn_id, status } => self.settle(turn_id, status),
        }
    }
}
