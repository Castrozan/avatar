use crate::session::client::{ClientId, Target};

pub(crate) use followers::{SpeechFollowers, speech_activity_message};
pub(crate) use roles::{ConnectedRole, connected_role};

use roles::{Memberships, first_held};

mod followers;
mod roles;

#[derive(Debug, Default)]
pub(crate) struct ConnectedClientRegistry {
    agents: Vec<ClientId>,
    renderer: Option<ClientId>,
    viewers: Vec<ClientId>,
    stage: Option<ClientId>,
    seat: Option<ClientId>,
}

impl ConnectedClientRegistry {
    pub(crate) fn register_agent(&mut self, client: ClientId) {
        add_unique(&mut self.agents, client);
    }

    pub(crate) fn register_viewer(&mut self, client: ClientId) {
        add_unique(&mut self.viewers, client);
    }

    pub(crate) fn register_renderer(&mut self, client: ClientId) {
        self.renderer = Some(client);
    }

    pub(crate) fn register_stage(&mut self, client: ClientId) {
        self.stage = Some(client);
    }

    pub(crate) fn register_seat(&mut self, client: ClientId) {
        self.seat = Some(client);
    }

    pub(crate) fn is_agent(&self, client: ClientId) -> bool {
        self.agents.contains(&client)
    }

    pub(crate) fn is_viewer(&self, client: ClientId) -> bool {
        self.viewers.contains(&client)
    }

    pub(crate) fn is_renderer(&self, client: ClientId) -> bool {
        self.renderer == Some(client)
    }

    pub(crate) fn is_stage(&self, client: ClientId) -> bool {
        self.stage == Some(client)
    }

    pub(crate) fn is_seat(&self, client: ClientId) -> bool {
        self.seat == Some(client)
    }

    pub(crate) fn has_renderer(&self) -> bool {
        self.renderer.is_some()
    }

    pub(crate) fn has_agents(&self) -> bool {
        !self.agents.is_empty()
    }

    pub(crate) fn camera_owner(&self) -> Option<ClientId> {
        self.stage.or(self.renderer)
    }

    pub(crate) fn forget(&mut self, client: ClientId) {
        let role = first_held(self.forget_precedence(client));
        self.drop_role(role, client);
    }

    pub(crate) fn recipients(&self, target: &Target) -> Vec<ClientId> {
        match target {
            Target::Client(client) => vec![*client],
            Target::Agents => self.agents.clone(),
            Target::Renderer => self.renderer.into_iter().collect(),
            Target::Viewers => self.viewers.clone(),
            Target::Stage => self.stage.into_iter().collect(),
            Target::DisplayCopies => self.viewers.iter().copied().chain(self.stage).collect(),
        }
    }

    fn forget_precedence(&self, client: ClientId) -> Memberships {
        [
            (ConnectedRole::Seat, self.is_seat(client)),
            (ConnectedRole::Stage, self.is_stage(client)),
            (ConnectedRole::Viewer, self.is_viewer(client)),
            (ConnectedRole::Agent, self.is_agent(client)),
            (ConnectedRole::Renderer, self.is_renderer(client)),
        ]
    }

    fn drop_role(&mut self, role: Option<ConnectedRole>, client: ClientId) {
        match role {
            Some(ConnectedRole::Seat) => self.seat = None,
            Some(ConnectedRole::Stage) => self.stage = None,
            Some(ConnectedRole::Viewer) => remove_client(&mut self.viewers, client),
            Some(ConnectedRole::Agent) => remove_client(&mut self.agents, client),
            Some(ConnectedRole::Renderer) => self.renderer = None,
            None => (),
        }
    }
}

fn remove_client(clients: &mut Vec<ClientId>, client: ClientId) {
    clients.retain(|other| *other != client);
}

fn add_unique(clients: &mut Vec<ClientId>, client: ClientId) {
    if !clients.contains(&client) {
        clients.push(client);
    }
}
