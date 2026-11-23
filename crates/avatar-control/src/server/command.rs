use tokio::sync::mpsc::UnboundedSender;

use crate::server::speech::SpeechEvent;
use crate::session::ClientId;

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
}
