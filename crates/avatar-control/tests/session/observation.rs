use avatar_control::session::{ClientId, Handoff, Hub, Outbound, Reply};
use serde_json::{Value, json};

#[derive(Default)]
pub(crate) struct Observation {
    pub(crate) outbound: Vec<Outbound>,
    pub(crate) closed: Vec<u64>,
    pub(crate) handoff: Option<Handoff>,
}

impl Observation {
    pub(crate) fn from_reply(client: ClientId, reply: Reply) -> Self {
        Self {
            outbound: reply.outbound,
            closed: reply
                .closes_sender
                .then_some(client.0)
                .into_iter()
                .collect(),
            handoff: reply.handoff,
        }
    }
}

pub(crate) fn observation_json(hub: &Hub, observation: &Observation) -> Value {
    json!({
        "sent": sent_messages(hub, &observation.outbound),
        "closed": observation.closed,
        "handoff": handoff_json(observation.handoff.as_ref()),
    })
}

pub(crate) fn sent_messages(hub: &Hub, outbound: &[Outbound]) -> Vec<Value> {
    outbound
        .iter()
        .flat_map(|item| {
            hub.recipients(&item.to)
                .into_iter()
                .map(move |client| json!([client.0, item.message.clone()]))
        })
        .collect()
}

pub(crate) fn handoff_json(handoff: Option<&Handoff>) -> Value {
    match handoff {
        None => Value::Null,
        Some(Handoff::StopSpeaking) => json!({"kind": "stopSpeaking"}),
        Some(Handoff::SelectVoice(name)) => json!({"kind": "selectVoice", "name": name}),
        Some(Handoff::Speak(_)) => json!({"kind": "speak"}),
    }
}
