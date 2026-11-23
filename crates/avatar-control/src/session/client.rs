use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClientId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Client(ClientId),
    Agents,
    Renderer,
    Viewers,
    Stage,
    DisplayCopies,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Outbound {
    pub to: Target,
    pub message: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Handoff {
    Speak(Value),
    StopSpeaking,
    SelectVoice(String),
}

#[derive(Debug, Default, PartialEq)]
pub struct Reply {
    pub outbound: Vec<Outbound>,
    pub handoff: Option<Handoff>,
    pub closes_sender: bool,
}

impl Reply {
    pub(crate) fn to_client(client: ClientId, message: Value) -> Self {
        Self::default().send(Target::Client(client), message)
    }

    pub(crate) fn send(mut self, to: Target, message: Value) -> Self {
        self.outbound.push(Outbound { to, message });
        self
    }

    pub(crate) fn hand_off(mut self, handoff: Handoff) -> Self {
        self.handoff = Some(handoff);
        self
    }

    pub(crate) fn close_sender(mut self) -> Self {
        self.closes_sender = true;
        self
    }
}
