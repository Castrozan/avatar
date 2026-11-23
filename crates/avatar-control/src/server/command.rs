use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::oneshot;

use crate::server::speech::SpeechEvent;
use crate::session::{ClientId, HealthSnapshot};

pub(crate) enum Command {
    Connect {
        client: ClientId,
        sender: UnboundedSender<String>,
    },
    Frame {
        client: ClientId,
        text: String,
    },
    Disconnect(ClientId),
    Tick,
    Speech(SpeechEvent),
    Health(oneshot::Sender<HealthSnapshot>),
    VoiceLevel(f64),
}
