mod events;
mod floor_turn;

use std::collections::VecDeque;

use self::floor_turn::FloorTurn;
use crate::speech::turn::{IllegalTurnEvent, TurnEvent, TurnPhase, next_turn_state};

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

    fn enqueue(&mut self, turn: EnqueuedTurn) -> Result<Vec<FloorEffect>, FloorError> {
        self.turns
            .push_back(FloorTurn::queued(turn.turn_id, turn.uses_agent_channel));
        match self.turns.len() {
            1 => self.advance(),
            _ => Ok(Vec::new()),
        }
    }

    fn advance(&mut self) -> Result<Vec<FloorEffect>, FloorError> {
        match self.turns.front().map(|front| front.state.phase) {
            None => Ok(Vec::new()),
            Some(TurnPhase::Ended) => self.complete_front(),
            Some(_) => self.begin_front(),
        }
    }

    fn begin_front(&mut self) -> Result<Vec<FloorEffect>, FloorError> {
        match self.turns.front_mut() {
            None => Ok(Vec::new()),
            Some(front) => Ok(vec![front.begin()?]),
        }
    }

    fn complete_front(&mut self) -> Result<Vec<FloorEffect>, FloorError> {
        match self.turns.pop_front() {
            None => Ok(Vec::new()),
            Some(completed) => self.settle(&completed),
        }
    }

    fn settle(&mut self, completed: &FloorTurn) -> Result<Vec<FloorEffect>, FloorError> {
        let mut effects = completed.release_effects();
        effects.push(FloorEffect::Settle {
            turn_id: completed.id,
            status: completed.state.status,
        });
        effects.extend(self.advance()?);
        Ok(effects)
    }

    fn agent_channel_opened(&mut self, turn_id: TurnId) -> Result<Vec<FloorEffect>, FloorError> {
        let turn = self.active_turn(turn_id)?;
        turn.agent_channel_open = true;
        Ok(vec![FloorEffect::StartSpeech(turn_id)])
    }

    fn speech_started(&mut self, turn_id: TurnId) -> Result<Vec<FloorEffect>, FloorError> {
        let turn = self.active_turn(turn_id)?;
        turn.state = next_turn_state(turn.state, TurnEvent::Synthesized)?;
        turn.has_playback = true;
        Ok(turn.stop_effects_when_ended())
    }

    fn finish_speech(
        &mut self,
        turn_id: TurnId,
        event: TurnEvent,
    ) -> Result<Vec<FloorEffect>, FloorError> {
        let turn = self.active_turn(turn_id)?;
        turn.state = next_turn_state(turn.state, event)?;
        turn.has_playback = false;
        self.complete_front()
    }

    fn interrupt(&mut self) -> Result<Vec<FloorEffect>, FloorError> {
        let outcomes = self
            .turns
            .iter_mut()
            .map(FloorTurn::interrupt)
            .collect::<Result<Vec<Option<FloorEffect>>, IllegalTurnEvent>>()?;
        Ok(outcomes.into_iter().flatten().collect())
    }

    fn active_turn(&mut self, turn_id: TurnId) -> Result<&mut FloorTurn, FloorError> {
        match self.turns.front_mut() {
            Some(turn) if turn.id == turn_id => Ok(turn),
            _ => Err(FloorError::NotActiveTurn(turn_id)),
        }
    }
}
