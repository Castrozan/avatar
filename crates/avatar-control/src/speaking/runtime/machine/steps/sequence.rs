use crate::speaking::runtime::machine::Runtime;
use crate::speech::chunk_sequence::{SequenceEffect, SequenceEvent, SpeechChunkSequence};
use crate::speech::floor::{FloorEvent, TurnId};
use crate::synthesis::contract::SpeechGenerator;

fn apply_all<Generator: SpeechGenerator + 'static>(
    runtime: &mut Runtime<Generator>,
    turn: TurnId,
    effects: Vec<SequenceEffect>,
) {
    for effect in effects {
        runtime.apply_sequence_effect(turn, effect);
    }
}

impl<Generator: SpeechGenerator + 'static> Runtime<Generator> {
    pub(crate) fn start_speech(&mut self, turn: TurnId) {
        let count = self
            .state
            .turns
            .get(&turn)
            .map(|record| record.chunks.len());
        count
            .map(SpeechChunkSequence::begin)
            .into_iter()
            .for_each(|(sequence, effects)| self.keep_sequence(turn, sequence, effects));
    }

    fn keep_sequence(
        &mut self,
        turn: TurnId,
        sequence: SpeechChunkSequence,
        effects: Vec<SequenceEffect>,
    ) {
        let mut sequence = Some(sequence);
        self.state
            .turns
            .get_mut(&turn)
            .into_iter()
            .for_each(|record| record.progress.sequence = sequence.take());
        self.run_effects(turn, effects);
    }

    pub(crate) fn sequence_event(&mut self, turn: TurnId, event: SequenceEvent) {
        let effects = self.sequence_effects(turn, event);
        self.run_effects(turn, effects);
    }

    pub(crate) fn sequence_effects(
        &mut self,
        turn: TurnId,
        event: SequenceEvent,
    ) -> Vec<SequenceEffect> {
        let record = self.state.turns.get_mut(&turn);
        let sequence = record.and_then(|record| record.progress.sequence.as_mut());
        sequence
            .map(|sequence| sequence.handle(event))
            .unwrap_or_default()
    }

    pub(crate) fn run_effects(&mut self, turn: TurnId, effects: Vec<SequenceEffect>) {
        apply_all(self, turn, effects);
    }

    fn apply_sequence_effect(&mut self, turn: TurnId, effect: SequenceEffect) {
        match effect {
            SequenceEffect::Synthesize(index) => self.synthesize(turn, index),
            SequenceEffect::Play(index) => self.play(turn, index),
            SequenceEffect::StopPlayback => self.stop_playback(turn),
            SequenceEffect::Finished => self.floor_event(FloorEvent::SpeechFinished(turn)),
            SequenceEffect::Failed => self.sequence_failed(turn),
        }
    }

    fn sequence_failed(&mut self, turn: TurnId) {
        let record = self.state.turns.get_mut(&turn);
        let message = record.and_then(|record| record.progress.chunk_error.take());
        self.fail_turn(turn, &message.unwrap_or_default());
    }
}
