use avatar_listening::conversation_state::{ConversationPhase, ListenerPurpose, observe};
use avatar_listening::listener::session::{
    SessionRequest, begin_listening, live_purpose, stop_listening,
};
use avatar_listening::listener::units::SessionUnits;
use serde_json::{Map, Value, json};

use super::MacRoom;
use crate::edge::failure::os_failure;
use crate::failure::{Failure, FailureKind};
use crate::session::{ListeningPlan, RoomSession};

const CONVERSATION_TARGET: &str = "avatar-conversation.target";

fn sources_of(plan: &ListeningPlan) -> Map<String, Value> {
    let named = plan.sources.iter();
    named
        .map(|(name, device)| (name.clone(), json!(device)))
        .collect()
}

fn settings_of(plan: &ListeningPlan) -> Map<String, Value> {
    let mut settings = Map::new();
    settings.insert("language".to_owned(), json!(plan.language));
    if plan.identify_speakers {
        settings.insert("identify_speakers".to_owned(), json!(true));
    }
    if plan.barge_in {
        settings.insert("barge_in".to_owned(), json!(true));
    }
    settings
}

fn session_failure(error: &impl ToString) -> Failure {
    Failure::new(FailureKind::Runtime, &error.to_string())
}

impl<Talk> RoomSession for MacRoom<Talk> {
    fn phase(&mut self) -> ConversationPhase {
        let active = self.session.units.is_active(CONVERSATION_TARGET);
        observe(active, live_purpose_of(self))
    }

    fn begin_listening(&mut self, plan: &ListeningPlan) -> Result<(), Failure> {
        let request = SessionRequest {
            purpose: plan.purpose,
            sources: sources_of(plan),
            settings: settings_of(plan),
        };
        begin_listening(&self.session.directory, &mut self.session.units, &request)
            .map_err(|error| session_failure(&error))
    }

    fn stop_listening(&mut self) -> Result<(), Failure> {
        stop_listening(&mut self.session.units).map_err(|error| session_failure(&error))
    }

    fn erase_session_records(&mut self) -> Result<(), Failure> {
        self.session
            .directory
            .erase_records()
            .map_err(|error| os_failure(&error))
    }
}

fn live_purpose_of<Talk>(room: &mut MacRoom<Talk>) -> Option<ListenerPurpose> {
    live_purpose(&room.session.directory, &mut room.session.units)
}
