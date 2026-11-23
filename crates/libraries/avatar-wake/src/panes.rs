use avatar_routing::names::session::Session;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LivePane {
    pub pane_id: String,
    pub servant: Option<String>,
}

#[must_use]
pub fn live_sessions(panes: &[LivePane]) -> Vec<Session> {
    panes.iter().filter_map(live_session).collect()
}

fn live_session(pane: &LivePane) -> Option<Session> {
    let servant = pane.servant.as_deref().filter(|name| !name.is_empty())?;
    Some(Session {
        servant: servant.to_owned(),
        pane: pane.pane_id.clone(),
        host: None,
    })
}
