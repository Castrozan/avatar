use std::time::Duration;

use serde_json::Value;

use crate::session::agent_commands;
use crate::session::client::{ClientId, Outbound, Reply, Target};
use crate::session::connection::{events, identify};
use crate::session::messages;
use crate::session::registry::{
    ConnectedClientRegistry, ConnectedRole, SpeechFollowers, connected_role,
};
use crate::session::state::AvatarState;
use crate::session::voice_levels::VoiceLevels;

const NULL_FRAME_FAULT: &str =
    "Message processing error: Cannot read properties of null (reading 'type')";
const IDENTIFY_REQUEST: &str =
    "Please identify with { type: \"identify\", role: \"agent\"|\"renderer\" }";

pub struct Hub {
    pub(crate) registry: ConnectedClientRegistry,
    pub(crate) followers: SpeechFollowers,
    pub(crate) avatar: AvatarState,
    pub(crate) voices: VoiceLevels,
    pub(crate) speech_phase: String,
    pub(crate) known_voices: Vec<String>,
}

impl Hub {
    #[must_use]
    pub fn new(started_at_ms: u64, known_voices: Vec<String>) -> Self {
        Self {
            registry: ConnectedClientRegistry::default(),
            followers: SpeechFollowers::default(),
            avatar: AvatarState::new(started_at_ms),
            voices: VoiceLevels::default(),
            speech_phase: "idle".to_owned(),
            known_voices,
        }
    }

    pub fn set_speech_phase(&mut self, phase: &str) {
        phase.clone_into(&mut self.speech_phase);
    }

    pub fn receive(&mut self, client: ClientId, frame: &str, now_ms: u64) -> Reply {
        receive_frame(self, client, frame, now_ms)
    }

    pub fn disconnect(&mut self, client: ClientId) {
        disconnect_client(self, client);
    }

    #[must_use]
    pub fn announce_speech(&mut self, speaking: bool) -> Vec<Outbound> {
        self.followers.announce(speaking)
    }

    pub fn tick_voice_levels(&mut self, now_ms: u64) -> Vec<Outbound> {
        self.voices.tick(Duration::from_millis(now_ms))
    }

    #[must_use]
    pub fn recipients(&self, target: &Target) -> Vec<ClientId> {
        self.registry.recipients(target)
    }
}

fn receive_frame(hub: &mut Hub, client: ClientId, frame: &str, now_ms: u64) -> Reply {
    match serde_json::from_str::<Value>(frame) {
        Ok(message) => route(hub, client, &message, now_ms),
        Err(parse_error) => Reply::to_client(
            client,
            messages::error(&format!("Message processing error: {parse_error}"), now_ms),
        ),
    }
}

fn disconnect_client(hub: &mut Hub, client: ClientId) {
    hub.followers.forget(client);
    if hub.registry.is_seat(client) {
        hub.voices.forget_delay();
    }
    hub.registry.forget(client);
}

fn route(hub: &mut Hub, client: ClientId, message: &Value, now_ms: u64) -> Reply {
    if message.is_null() {
        return Reply::to_client(client, messages::error(NULL_FRAME_FAULT, now_ms));
    }
    if message.get("type") == Some(&Value::from("identify")) {
        return identify::identify(hub, client, message.get("role"), now_ms);
    }
    route_identified(hub, client, message, now_ms)
}

fn route_identified(hub: &mut Hub, client: ClientId, message: &Value, now_ms: u64) -> Reply {
    match connected_role(&hub.registry, client) {
        Some(ConnectedRole::Seat) => events::seat_event(hub, message),
        Some(ConnectedRole::Agent) => agent_commands::route(hub, client, message, now_ms),
        Some(ConnectedRole::Renderer) => events::renderer_event(hub, message),
        Some(ConnectedRole::Stage) => events::stage_event(message),
        Some(ConnectedRole::Viewer) => Reply::default(),
        None => identify_request(client),
    }
}

fn identify_request(client: ClientId) -> Reply {
    let request = messages::message(
        "identifyRequest",
        vec![("message", Some(Value::from(IDENTIFY_REQUEST)))],
    );
    Reply::to_client(client, request)
}
