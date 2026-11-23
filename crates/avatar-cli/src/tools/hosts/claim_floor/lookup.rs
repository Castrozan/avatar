use avatar_adapters::commands::{A2aLookup, Programs, TalkProcess};
use avatar_conversation::talk::ports::AgentPane;
use avatar_floor::floor::RemoteAgent;
use std::cell::RefCell;

pub(super) fn servant_at(agent: &RemoteAgent) -> Result<Option<String>, String> {
    let mut lookup = A2aLookup::new(TalkProcess::new(Programs::on_path()));
    let pane = AgentPane {
        host: agent.host.clone(),
        pane: Some(agent.pane.clone()),
    };
    lookup
        .servant_at(&pane)
        .map_err(|failure| failure.to_string())
}

#[derive(Default)]
pub(super) struct RememberedLookup {
    failure: RefCell<Option<String>>,
}

impl RememberedLookup {
    pub(super) fn servant_at(&self, agent: &RemoteAgent) -> Option<String> {
        servant_at(agent).unwrap_or_else(|message| self.remember(message))
    }

    fn remember(&self, message: String) -> Option<String> {
        *self.failure.borrow_mut() = Some(message);
        None
    }

    pub(super) fn failure(&self) -> Option<String> {
        self.failure.borrow().clone()
    }
}
