use super::Runtime;
use crate::server::SpeechEvent;
use crate::speech::turn::TurnPhase;
use crate::synthesis::contract::SpeechGenerator;

pub(super) const IDLE: &str = "idle";

fn phase_name(phase: Option<TurnPhase>) -> &'static str {
    match phase {
        Some(TurnPhase::Synthesizing) => "synthesizing",
        Some(TurnPhase::Playing) => "playing",
        _ => IDLE,
    }
}

impl<Generator: SpeechGenerator + 'static> Runtime<Generator> {
    pub(super) fn publish_phase(&mut self) {
        let phase = phase_name(self.state.floor.current_phase());
        (phase != self.phase)
            .then_some(phase)
            .into_iter()
            .for_each(|name| self.change_phase(name));
    }

    fn change_phase(&mut self, name: &'static str) {
        self.phase = name;
        self.emit(SpeechEvent::Phase(name.to_owned()));
    }
}
