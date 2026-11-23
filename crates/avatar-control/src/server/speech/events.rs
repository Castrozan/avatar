use tokio::sync::mpsc::{Sender, UnboundedReceiver};

use super::port::SpeechEvent;
use crate::server::command::Command;
use crate::session::{Hub, Outbound};

pub(crate) fn outbound_for(hub: &mut Hub, event: SpeechEvent) -> Vec<Outbound> {
    match event {
        SpeechEvent::Deliver(outbound) => vec![outbound],
        SpeechEvent::Activity(speaking) => hub.announce_speech(speaking),
        SpeechEvent::Phase(phase) => {
            hub.set_speech_phase(&phase);
            Vec::new()
        }
    }
}

pub(crate) async fn forward_events(
    mut events: UnboundedReceiver<SpeechEvent>,
    commands: Sender<Command>,
) {
    while let Some(event) = events.recv().await {
        drop(commands.send(Command::Speech(event)).await);
    }
}
