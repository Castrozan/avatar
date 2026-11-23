use crate::session::client::{ClientId, Reply};
use crate::session::hub::Hub;
use crate::session::registry::speech_activity_message;

pub(crate) fn follow_speech_command(hub: &mut Hub, client: ClientId, kind: &str) -> Option<Reply> {
    match kind {
        "followSpeech" => Some(follow_speech(hub, client)),
        _ => None,
    }
}

fn follow_speech(hub: &mut Hub, client: ClientId) -> Reply {
    hub.followers.follow(client);
    Reply::to_client(client, speech_activity_message(hub.followers.is_speaking()))
}
