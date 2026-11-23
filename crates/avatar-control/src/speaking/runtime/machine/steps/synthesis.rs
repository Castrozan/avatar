use std::sync::Arc;

use tokio::sync::mpsc::UnboundedSender;

use crate::speaking::runtime::machine::input::ChunkSynthesized;
use crate::speaking::runtime::machine::{Input, Runtime};
use crate::speech::chunk_sequence::SequenceEvent;
use crate::speech::floor::{FloorEvent, TurnId};
use crate::synthesis::contract::{GeneratedSpeech, SpeechGenerator, SpeechRequest};

struct ChunkAddress {
    turn: TurnId,
    index: usize,
}

async fn synthesize_and_report<Generator: SpeechGenerator>(
    generator: Arc<Generator>,
    request: SpeechRequest,
    address: ChunkAddress,
    inputs: UnboundedSender<Input>,
) {
    let result = generator.synthesize(request).await;
    drop(inputs.send(Input::ChunkSynthesized(ChunkSynthesized {
        turn: address.turn,
        index: address.index,
        result,
    })));
}

impl<Generator: SpeechGenerator + 'static> Runtime<Generator> {
    pub(crate) fn synthesize(&mut self, turn: TurnId, index: usize) {
        let now = self.services.clock.now_ms();
        let record = self.state.turns.get_mut(&turn);
        record
            .map(|record| record.begin_synthesis(index, now))
            .into_iter()
            .for_each(|request| self.spawn_synthesis(request, ChunkAddress { turn, index }));
    }

    fn spawn_synthesis(&self, request: SpeechRequest, address: ChunkAddress) {
        let generator = Arc::clone(&self.generator);
        tokio::spawn(synthesize_and_report(
            generator,
            request,
            address,
            self.inputs.clone(),
        ));
    }

    pub(crate) fn chunk_synthesized(&mut self, synthesized: ChunkSynthesized) {
        let ChunkSynthesized {
            turn,
            index,
            result,
        } = synthesized;
        match result {
            Ok(speech) => self.chunk_ready(turn, index, speech),
            Err(fault) => self.chunk_failed(turn, &fault.to_string()),
        }
    }

    fn chunk_failed(&mut self, turn: TurnId, message: &str) {
        self.state
            .turns
            .get_mut(&turn)
            .into_iter()
            .for_each(|record| record.progress.chunk_error = Some(message.to_owned()));
        self.sequence_event(turn, SequenceEvent::ChunkFailed);
    }

    fn chunk_ready(&mut self, turn: TurnId, index: usize, speech: GeneratedSpeech) {
        let now = self.services.clock.now_ms();
        let record = self.state.turns.get_mut(&turn);
        record
            .map(|record| record.record_synthesis(index, speech, now))
            .into_iter()
            .for_each(|()| self.announce_ready(turn, index));
    }

    fn announce_ready(&mut self, turn: TurnId, index: usize) {
        match index {
            0 => self.first_chunk_ready(turn),
            _ => self.sequence_event(turn, SequenceEvent::ChunkReady(index)),
        }
    }

    fn first_chunk_ready(&mut self, turn: TurnId) {
        let holds_floor = self.state.floor.holds_floor(turn);
        let effects = self.sequence_effects(turn, SequenceEvent::FirstChunkReady { holds_floor });
        self.floor_event(FloorEvent::SpeechStarted(turn));
        self.run_effects(turn, effects);
    }
}
