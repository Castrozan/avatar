use avatar_listening::conversation_state::{
    ConversationPhase, ListenerProbe, ListenerPurpose, live_purpose as probed_purpose, observe,
};

use super::CONVERSATION_TARGET;
use super::configuration::read_configuration;
use crate::talk::ports::{Files, Units};

struct Probe<'boundary, Boundary>(&'boundary mut Boundary);

impl<Boundary: Units + Files> ListenerProbe for Probe<'_, Boundary> {
    fn unit_active(&mut self, unit: &str) -> bool {
        self.0.unit_active(unit)
    }

    fn configured_purpose(&mut self) -> Option<String> {
        let configuration = read_configuration(self.0).ok()?;
        configuration.get("purpose")?.as_str().map(str::to_owned)
    }
}

pub fn live_purpose(boundary: &mut (impl Units + Files)) -> Option<ListenerPurpose> {
    probed_purpose(&mut Probe(boundary))
}

pub fn phase(boundary: &mut (impl Units + Files)) -> ConversationPhase {
    let target_active = boundary.unit_active(CONVERSATION_TARGET);
    observe(target_active, live_purpose(boundary))
}
