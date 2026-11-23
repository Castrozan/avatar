use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnPhase {
    Queued,
    Synthesizing,
    Playing,
    Ended,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnEvent {
    Begin,
    Synthesized,
    Finish,
    Interrupt,
    Fail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnStatus {
    Finished,
    Interrupted,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TurnState {
    pub phase: TurnPhase,
    pub status: Option<TurnStatus>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IllegalTurnEvent {
    pub event: TurnEvent,
    pub phase: TurnPhase,
}

impl fmt::Display for IllegalTurnEvent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "speech turn cannot take {:?} while {:?}",
            self.event, self.phase
        )
    }
}

impl Error for IllegalTurnEvent {}

pub fn next_turn_state(state: TurnState, event: TurnEvent) -> Result<TurnState, IllegalTurnEvent> {
    match state.phase {
        TurnPhase::Queued => queued_transition(state, event),
        TurnPhase::Synthesizing => synthesizing_transition(state, event),
        TurnPhase::Playing => playing_transition(state, event),
        TurnPhase::Ended => Ok(state),
    }
}

fn queued_transition(state: TurnState, event: TurnEvent) -> Result<TurnState, IllegalTurnEvent> {
    match event {
        TurnEvent::Begin => Ok(turn_state(TurnPhase::Synthesizing, None)),
        TurnEvent::Interrupt => Ok(ended(TurnStatus::Cancelled)),
        TurnEvent::Fail => Ok(ended(TurnStatus::Failed)),
        TurnEvent::Synthesized | TurnEvent::Finish => Err(illegal(state, event)),
    }
}

fn synthesizing_transition(
    state: TurnState,
    event: TurnEvent,
) -> Result<TurnState, IllegalTurnEvent> {
    match event {
        TurnEvent::Synthesized => Ok(turn_state(TurnPhase::Playing, None)),
        TurnEvent::Interrupt => Ok(ended(TurnStatus::Interrupted)),
        TurnEvent::Fail => Ok(ended(TurnStatus::Failed)),
        TurnEvent::Begin | TurnEvent::Finish => Err(illegal(state, event)),
    }
}

fn playing_transition(state: TurnState, event: TurnEvent) -> Result<TurnState, IllegalTurnEvent> {
    match event {
        TurnEvent::Finish => Ok(ended(TurnStatus::Finished)),
        TurnEvent::Interrupt => Ok(ended(TurnStatus::Interrupted)),
        TurnEvent::Fail => Ok(ended(TurnStatus::Failed)),
        TurnEvent::Begin | TurnEvent::Synthesized => Err(illegal(state, event)),
    }
}

const fn turn_state(phase: TurnPhase, status: Option<TurnStatus>) -> TurnState {
    TurnState { phase, status }
}

const fn ended(status: TurnStatus) -> TurnState {
    turn_state(TurnPhase::Ended, Some(status))
}

const fn illegal(state: TurnState, event: TurnEvent) -> IllegalTurnEvent {
    IllegalTurnEvent {
        event,
        phase: state.phase,
    }
}
