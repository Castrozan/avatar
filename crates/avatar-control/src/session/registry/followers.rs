use serde_json::Value;

use crate::session::client::{ClientId, Outbound, Target};
use crate::session::messages::message;

#[derive(Debug, Default)]
pub(crate) struct SpeechFollowers {
    clients: Vec<ClientId>,
    speaking: bool,
}

impl SpeechFollowers {
    pub(crate) fn follow(&mut self, client: ClientId) {
        add_unique(&mut self.clients, client);
    }

    pub(crate) fn is_speaking(&self) -> bool {
        self.speaking
    }

    pub(crate) fn forget(&mut self, client: ClientId) {
        self.clients.retain(|other| *other != client);
    }

    pub(crate) fn announce(&mut self, speaking: bool) -> Vec<Outbound> {
        self.speaking = speaking;
        self.clients
            .iter()
            .map(|client| Outbound {
                to: Target::Client(*client),
                message: speech_activity_message(speaking),
            })
            .collect()
    }
}

pub(crate) fn speech_activity_message(speaking: bool) -> Value {
    message(
        "speechActivity",
        vec![("speaking", Some(Value::from(speaking)))],
    )
}

fn add_unique(clients: &mut Vec<ClientId>, client: ClientId) {
    if !clients.contains(&client) {
        clients.push(client);
    }
}
