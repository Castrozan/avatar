mod events;
mod floor_turn;
mod transitions;

use std::collections::VecDeque;

use self::floor_turn::FloorTurn;
use crate::speech::turn::{TurnEvent, TurnPhase};

pub use events::{EnqueuedTurn, FloorEffect, FloorError, FloorEvent, TurnId};

#[derive(Debug, Default)]
pub struct SpeechFloor {
    turns: VecDeque<FloorTurn>,
}

impl SpeechFloor {
    pub fn handle(&mut self, event: FloorEvent) -> Result<Vec<FloorEffect>, FloorError> {
        match event {
            FloorEvent::Enqueue(turn) => self.enqueue(turn),
            FloorEvent::AgentChannelOpened(turn_id) => self.agent_channel_opened(turn_id),
            FloorEvent::SpeechStarted(turn_id) => self.speech_started(turn_id),
            FloorEvent::SpeechFinished(turn_id) => self.finish_speech(turn_id, TurnEvent::Finish),
            FloorEvent::SpeechFailed(turn_id) => self.finish_speech(turn_id, TurnEvent::Fail),
            FloorEvent::Interrupt => self.interrupt(),
        }
    }

    #[must_use]
    pub fn current_phase(&self) -> Option<TurnPhase> {
        self.turns
            .iter()
            .map(|turn| turn.state.phase)
            .find(|phase| matches!(phase, TurnPhase::Synthesizing | TurnPhase::Playing))
    }

    #[must_use]
    pub fn holds_floor(&self, turn_id: TurnId) -> bool {
        self.turns
            .iter()
            .any(|turn| turn.id == turn_id && turn.state.phase != TurnPhase::Ended)
    }
}
