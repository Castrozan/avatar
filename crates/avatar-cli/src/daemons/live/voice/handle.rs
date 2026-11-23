use avatar_live::boundaries::VoiceDelivery;
use avatar_live::client::GptLiveClient;
use avatar_live::events::LiveEvent;
use tokio::sync::{mpsc, oneshot};

pub(crate) struct Delivery {
    pub(crate) task_id: String,
    pub(crate) text: String,
    pub(crate) is_final: bool,
}

pub(crate) struct Change {
    pub(crate) voice: String,
    pub(crate) done: oneshot::Sender<()>,
}

pub(crate) struct Replacement {
    pub(crate) client: GptLiveClient,
    pub(crate) voice: String,
    pub(crate) done: oneshot::Sender<()>,
}

pub(crate) enum Outbound {
    Audio(Vec<u8>),
    Deliver(Delivery),
    Close,
}

pub(crate) enum VoiceCommand {
    Send(Outbound),
    Reconnect(Reconnection),
}

pub(crate) enum Reconnection {
    Change(Change),
    Replace(Replacement),
    Failed(String),
}

pub(crate) enum Occurrence {
    Event(LiveEvent),
    VoiceChanged,
}

#[derive(Clone)]
pub(crate) struct VoiceHandle {
    commands: mpsc::UnboundedSender<VoiceCommand>,
}

pub(crate) struct ActorLine {
    pub(crate) commands: mpsc::UnboundedReceiver<VoiceCommand>,
    pub(crate) own_commands: mpsc::UnboundedSender<VoiceCommand>,
    pub(crate) occurrences: mpsc::UnboundedSender<Occurrence>,
}

pub(crate) fn voice_line() -> (VoiceHandle, ActorLine, mpsc::UnboundedReceiver<Occurrence>) {
    let (commands, received_commands) = mpsc::unbounded_channel();
    let (occurrences, received) = mpsc::unbounded_channel();
    let line = ActorLine {
        commands: received_commands,
        own_commands: commands.clone(),
        occurrences,
    };
    (VoiceHandle { commands }, line, received)
}

impl VoiceHandle {
    pub(crate) fn send_audio(&self, pcm: Vec<u8>) {
        let _ = self.commands.send(VoiceCommand::Send(Outbound::Audio(pcm)));
    }

    pub(crate) fn close(&self) {
        let _ = self.commands.send(VoiceCommand::Send(Outbound::Close));
    }

    pub(crate) async fn change_to(&self, voice: String) {
        let (done, finished) = oneshot::channel();
        let _ = self
            .commands
            .send(VoiceCommand::Reconnect(Reconnection::Change(Change {
                voice,
                done,
            })));
        let _ = finished.await;
    }
}

impl VoiceDelivery for VoiceHandle {
    async fn deliver(&mut self, task_id: &str, result: &str, is_final: bool) {
        let delivery = Delivery {
            task_id: task_id.to_owned(),
            text: result.to_owned(),
            is_final,
        };
        let _ = self
            .commands
            .send(VoiceCommand::Send(Outbound::Deliver(delivery)));
    }
}
