use crate::session::client::ClientId;
use crate::session::registry::ConnectedClientRegistry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ConnectedRole {
    Seat,
    Agent,
    Renderer,
    Stage,
    Viewer,
}

pub(crate) type Memberships = [(ConnectedRole, bool); 5];

pub(crate) fn first_held(memberships: Memberships) -> Option<ConnectedRole> {
    memberships
        .into_iter()
        .find(|(_, held)| *held)
        .map(|(role, _)| role)
}

pub(crate) fn connected_role(
    registry: &ConnectedClientRegistry,
    client: ClientId,
) -> Option<ConnectedRole> {
    first_held([
        (ConnectedRole::Seat, registry.is_seat(client)),
        (ConnectedRole::Agent, registry.is_agent(client)),
        (ConnectedRole::Renderer, registry.is_renderer(client)),
        (ConnectedRole::Stage, registry.is_stage(client)),
        (ConnectedRole::Viewer, registry.is_viewer(client)),
    ])
}
