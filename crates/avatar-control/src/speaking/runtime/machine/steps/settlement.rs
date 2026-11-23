use serde_json::Value;

use crate::server::SpeechEvent;
use crate::session::{ClientId, Outbound, Target};
use crate::speaking::runtime::machine::Runtime;
use crate::speaking::runtime::machine::turn::Turn;
use crate::speaking::{error_reply, speak_ack, speech_ended};
use crate::speech::floor::TurnId;
use crate::speech::turn::TurnStatus;
use crate::synthesis::contract::SpeechGenerator;

fn settlement_message(record: &Turn, status: Option<TurnStatus>, now_ms: u64) -> Value {
    match &record.progress.failure {
        Some(failure) => error_reply(&format!("Speech failed: {failure}"), now_ms),
        None => speak_ack(
            &record.request,
            status.unwrap_or(TurnStatus::Failed),
            &record.progress.sources,
            &record.progress.timing,
        ),
    }
}

impl<Generator: SpeechGenerator + 'static> Runtime<Generator> {
    pub(crate) fn settle(&mut self, turn: TurnId, status: Option<TurnStatus>) {
        self.state
            .turns
            .remove(&turn)
            .into_iter()
            .for_each(|record| self.deliver_settlement(&record, status));
    }

    fn deliver_settlement(&self, record: &Turn, status: Option<TurnStatus>) {
        self.state
            .turns
            .is_empty()
            .then_some(())
            .into_iter()
            .for_each(|()| self.emit(SpeechEvent::Activity(false)));
        let now = self.services.clock.now_ms();
        self.reply(record.client, settlement_message(record, status, now));
        self.emit_to_renderer(speech_ended(&record.request));
    }

    pub(crate) fn reply(&self, client: ClientId, message: Value) {
        self.deliver(Target::Client(client), message);
    }

    pub(crate) fn emit_to_renderer(&self, message: Value) {
        self.deliver(Target::Renderer, message);
    }

    fn deliver(&self, to: Target, message: Value) {
        self.emit(SpeechEvent::Deliver(Outbound { to, message }));
    }
}
