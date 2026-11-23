use serde_json::Value;

use super::Runtime;
use super::turn::Turn;
use crate::server::SpeechEvent;
use crate::session::ClientId;
use crate::speaking::request::is_blank;
use crate::speaking::runtime::players::playback_targets;
use crate::speaking::{OutputPlan, SpeakRequest, error_reply, output_plan};
use crate::speech::floor::{EnqueuedTurn, FloorEvent, TurnId};
use crate::speech::speech_chunks::split_into_chunks;
use crate::synthesis::contract::SpeechGenerator;

const REFUSAL: &str = "Missing required field: text";
const MAXIMUM_CHUNK_CHARACTERS: usize = 200;

struct Order {
    client: ClientId,
    received_ms: u64,
}

fn speakable_chunks(text: &str) -> Result<Vec<String>, String> {
    let chunks =
        split_into_chunks(text, MAXIMUM_CHUNK_CHARACTERS).map_err(|fault| fault.to_string())?;
    Ok(chunks
        .into_iter()
        .filter(|chunk| !is_blank(chunk))
        .collect())
}

fn enqueue_event(turn_id: TurnId, plan: &OutputPlan) -> FloorEvent {
    FloorEvent::Enqueue(EnqueuedTurn {
        turn_id,
        uses_agent_channel: plan.uses_agent_channel,
    })
}

impl<Generator: SpeechGenerator + 'static> Runtime<Generator> {
    pub(super) fn speak(&mut self, client: ClientId, received_ms: u64, command: &Value) {
        let order = Order {
            client,
            received_ms,
        };
        match SpeakRequest::from_command(command, received_ms) {
            Some(request) => self.accept(&order, request),
            None => self.refuse(&order, REFUSAL),
        }
    }

    pub(super) fn interrupt(&mut self) {
        self.floor_event(FloorEvent::Interrupt);
    }

    fn accept(&mut self, order: &Order, request: SpeakRequest) {
        match speakable_chunks(&request.text) {
            Ok(chunks) if !chunks.is_empty() => self.enqueue(order, request, chunks),
            Ok(_) => self.refuse(order, REFUSAL),
            Err(fault) => self.refuse(order, &format!("Speech failed: {fault}")),
        }
    }

    fn refuse(&self, order: &Order, text: &str) {
        self.reply(order.client, error_reply(text, order.received_ms));
    }

    fn enqueue(&mut self, order: &Order, request: SpeakRequest, chunks: Vec<String>) {
        let plan = output_plan(&request.output);
        let targets = playback_targets(&plan.route, self.services.platform);
        let turn = Turn::new(order.client, request, chunks, targets, order.received_ms);
        let turn_id = self.state.next_turn;
        self.state.next_turn += 1;
        self.state.turns.insert(turn_id, turn);
        self.announce_first_turn();
        self.floor_event(enqueue_event(turn_id, &plan));
    }

    fn announce_first_turn(&self) {
        (self.state.turns.len() == 1)
            .then_some(())
            .into_iter()
            .for_each(|()| self.emit(SpeechEvent::Activity(true)));
    }
}
