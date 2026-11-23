use super::events::{FloorEffect, TurnId};
use crate::speech::turn::{IllegalTurnEvent, TurnEvent, TurnPhase, TurnState, next_turn_state};

#[derive(Debug)]
pub(super) struct FloorTurn {
    pub(super) id: TurnId,
    pub(super) state: TurnState,
    pub(super) uses_agent_channel: bool,
    pub(super) agent_channel_open: bool,
    pub(super) has_playback: bool,
}

impl FloorTurn {
    #[must_use]
    pub(super) const fn queued(id: TurnId, uses_agent_channel: bool) -> Self {
        Self {
            id,
            state: TurnState {
                phase: TurnPhase::Queued,
                status: None,
            },
            uses_agent_channel,
            agent_channel_open: false,
            has_playback: false,
        }
    }

    pub(super) fn begin(&mut self) -> Result<FloorEffect, IllegalTurnEvent> {
        self.state = next_turn_state(self.state, TurnEvent::Begin)?;
        Ok(self.entry_effect())
    }

    fn entry_effect(&self) -> FloorEffect {
        self.uses_agent_channel.then_some(self.id).map_or(
            FloorEffect::StartSpeech(self.id),
            FloorEffect::OpenAgentChannel,
        )
    }

    pub(super) fn release_effects(&self) -> Vec<FloorEffect> {
        self.agent_channel_open
            .then_some(FloorEffect::ReleaseAgentChannel(self.id))
            .into_iter()
            .collect()
    }

    pub(super) fn stop_effects_when_ended(&self) -> Vec<FloorEffect> {
        (self.state.phase == TurnPhase::Ended)
            .then_some(FloorEffect::StopPlayback(self.id))
            .into_iter()
            .collect()
    }

    pub(super) fn interrupt(&mut self) -> Result<Option<FloorEffect>, IllegalTurnEvent> {
        self.state = next_turn_state(self.state, TurnEvent::Interrupt)?;
        Ok(self
            .has_playback
            .then_some(FloorEffect::StopPlayback(self.id)))
    }
}
